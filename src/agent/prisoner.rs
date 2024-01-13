use std::fmt::{Display, Formatter};
use rand::seq::IteratorRandom;
use tch::Tensor;
use amfi_core::agent::{InformationSet, Policy, PresentPossibleActions, EvaluatedInformationSet};
use amfi_core::domain::{DomainParameters, Reward};
use amfi_core::error::ConvertError;
use amfi_rl::tensor_data::{ActionTensor, ConvertToTensor, ConversionToTensor};
use enum_map::Enum;
use amfi_rl::error::TensorRepresentationError;
use crate::domain::{ClassicAction, ClassicGameDomainNamed, ClassicGameError, ClassicGameUpdate, EncounterReportNamed, PrisonerId};
use crate::domain::ClassicAction::{Cooperate, Defect};
use crate::SymmetricRewardTableInt;

#[derive(Clone, Debug)]
pub struct PrisonerInfoSet {
    id: PrisonerId,
    previous_actions: Vec<EncounterReportNamed>,
    reward_table: SymmetricRewardTableInt,
    //last_action: Cell<Option<PrisonerAction>>

}

impl PrisonerInfoSet {
    pub fn new(player_id: PrisonerId, reward_table: SymmetricRewardTableInt) -> Self{
        Self{
            id: player_id,
            reward_table,
            //last_action: Cell::new(None),
            previous_actions: Vec::new()}
    }

    /*pub fn _select_action(&self, action: PrisonerAction){
        self.last_action.set(Some(action));
    }

     */

    pub fn previous_actions(&self) -> &Vec<EncounterReportNamed>{
        &self.previous_actions
    }

    pub fn count_actions(&self, action: ClassicAction) -> usize{
        self.previous_actions.iter().filter(|update| update.own_action == action)
            .count()
    }
}

impl Display for PrisonerInfoSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rounds: {} |", self.previous_actions.len())?;
        let mut s = self.previous_actions.iter().fold( String::new(),|mut acc, update|{
            acc.push_str(&format!("{:#}-{:#} ", update.own_action, update.other_player_action));
            acc
        });
        s.pop();
        write!(f, "{}", s)
    }
}

/*
pub struct CoverPolicy{}

impl Policy<ClassicGameDomain> for CoverPolicy{
    type InfoSetType = PrisonerInfoSet;

    fn select_action(&self, _state: &Self::InfoSetType) -> Option<ClassicAction> {
        //state._select_action(Cover);
        Some(Cooperate)
    }
}*/

pub struct Forgive1Policy{}

impl Policy<ClassicGameDomainNamed> for Forgive1Policy{
    type InfoSetType = PrisonerInfoSet;

    fn select_action(&self, state: &Self::InfoSetType) -> Option<ClassicAction> {
        let enemy_betrayals = state.previous_actions().iter().filter(| &step|{
            step.other_player_action == Defect
        }).count();
        if enemy_betrayals > 1 {
            //state._select_action(Betray);
            Some(Defect)
        } else {
            //state._select_action(Cover);
            Some(Cooperate)
        }

    }
}

pub struct BetrayRatioPolicy{}

impl Policy<ClassicGameDomainNamed> for BetrayRatioPolicy{
    type InfoSetType = PrisonerInfoSet;

    fn select_action(&self, state: &Self::InfoSetType) -> Option<ClassicAction> {
        let betrayed = state.previous_actions().iter()
            .filter(|round| round.other_player_action == Defect)
            .count();
        let covered = state.previous_actions().iter()
            .filter(|round| round.other_player_action == Cooperate)
            .count();

        if betrayed > covered{
            //state._select_action(Betray);
            Some(Defect)
        } else {
            //state._select_action(Cover);
            Some(Cooperate)
        }
    }
}


pub struct RandomPrisonerPolicy{}


impl Policy<ClassicGameDomainNamed> for RandomPrisonerPolicy{
    type InfoSetType = PrisonerInfoSet;


    fn select_action(&self, state: &Self::InfoSetType) -> Option<ClassicAction> {
        let mut rng = rand::thread_rng();
        state.available_actions().into_iter().choose(&mut rng)


    }
}


pub struct SwitchOnTwoSubsequent{}

impl Policy<ClassicGameDomainNamed> for SwitchOnTwoSubsequent{
    type InfoSetType = PrisonerInfoSet;

    fn select_action(&self, state: &Self::InfoSetType) -> Option<ClassicAction> {

        if let Some(i_update) = state.previous_actions().last(){
            let mut other_action = i_update.other_player_action;
            for i in (0..state.previous_actions.len()-1).rev(){
                if state.previous_actions()[i].other_player_action == other_action{
                    return Some(other_action)
                } else {
                    other_action = state.previous_actions()[i].other_player_action;
                }
            }
            Some(Cooperate)
        } else{
            Some(Cooperate)
        }

    }
}

impl InformationSet<ClassicGameDomainNamed> for PrisonerInfoSet {
    fn agent_id(&self) -> &<ClassicGameDomainNamed as DomainParameters>::AgentId {
        &self.id
    }

    fn is_action_valid(&self, _action: &ClassicAction) -> bool {
        true
    }

    fn update(&mut self, update: ClassicGameUpdate<PrisonerId>) -> Result<(), ClassicGameError<PrisonerId>> {

        let encounter = update.encounters[self.id.into_usize()];
        self.previous_actions.push(encounter);
        Ok(())
    }

}

impl PresentPossibleActions<ClassicGameDomainNamed> for PrisonerInfoSet {
    type ActionIteratorType = [ClassicAction;2];

    fn available_actions(&self) -> Self::ActionIteratorType {
        [Defect, Cooperate]
    }
}

impl EvaluatedInformationSet<ClassicGameDomainNamed> for PrisonerInfoSet {
    type RewardType = f64;

    fn current_subjective_score(&self) -> Self::RewardType {
        if !self.previous_actions.is_empty(){
            let sum = self.previous_actions.iter().fold(0.0, |acc, x|{
                acc + self.reward_table.reward(x.own_action, x.other_player_action) as f64
            });
            sum/(self.previous_actions.len() as f64)

        } else{
            Self::RewardType::neutral()
        }



        //self.previous_actions.len() as f64
    }

    fn penalty_for_illegal(&self) -> Self::RewardType {
        -100.0
    }
}

pub struct PrisonerStateTranslate{

}
/*
impl ConvStateToTensor<PrisonerInfoSet> for PrisonerStateTranslate{
    fn make_tensor(&self, t: &PrisonerInfoSet) -> Tensor {
        let mut array = [0.0f32;2*256];
        for i in 0..t.previous_actions().len(){
            array[2*i] = match t.previous_actions()[i].own_action{
                Defect =>  1.0,
                Cooperate => 2.0,
            };
            array[2*i+1] = match t.previous_actions()[i].other_player_action {
                Defect =>  1.0,
                Cooperate => 2.0,
            };
        }
        Tensor::from_slice(&array[..])
    }
}

 */



#[derive(Default)]
pub struct PrisonerInfoSetWay{}
const PRISONER_INFOSET_SHAPE: [i64;1] = [512];
impl ConversionToTensor for PrisonerInfoSetWay{
    fn desired_shape(&self) -> &'static [i64] {
        &PRISONER_INFOSET_SHAPE[..]
    }
}

impl ConvertToTensor<PrisonerInfoSetWay> for PrisonerInfoSet {
    fn try_to_tensor(&self, _way: &PrisonerInfoSetWay) -> Result<Tensor, TensorRepresentationError> {
        let mut array = [0.0f32;2*256];
        for i in 0..self.previous_actions().len(){
            array[2*i] = match self.previous_actions()[i].own_action{
                Defect =>  1.0,
                Cooperate => 2.0,
            };
            array[2*i+1] = match self.previous_actions()[i].other_player_action {
                Defect =>  1.0,
                Cooperate => 2.0,
            };
        }
        Ok(Tensor::from_slice(&array[..]))
    }
}