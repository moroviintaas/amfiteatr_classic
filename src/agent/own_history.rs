use std::fmt::{Display, Formatter};
use log::trace;
use tch::Tensor;
use amfi::agent::{AgentIdentifier, InformationSet, PresentPossibleActions, ScoringInformationSet};
use amfi::domain::{Renew};
use amfi_rl::error::TensorRepresentationError;
use amfi_rl::tensor_repr::{ConvertToTensor, WayToTensor};
use crate::AsymmetricRewardTableInt;
use crate::domain::{AgentNum, AsUsize, ClassicAction, ClassicGameDomain, ClassicGameError, ClassicGameUpdate, EncounterReport, UsizeAgentId};

#[derive(Clone, Debug)]
pub struct OwnHistoryInfoSet<ID: AgentIdentifier>{
    id: ID,
    previous_encounters: Vec<EncounterReport<ID>>,
    reward_table: AsymmetricRewardTableInt

}

impl<ID: AgentIdentifier> OwnHistoryInfoSet<ID>{

    pub fn new(id: ID, reward_table: AsymmetricRewardTableInt) -> Self{
        Self{id, reward_table, previous_encounters: Default::default()}
    }

    pub fn reset(&mut self){
        self.previous_encounters.clear();
    }

    pub fn previous_encounters(&self) -> &Vec<EncounterReport<ID>>{
        &self.previous_encounters
    }

    pub fn count_actions_self(&self, action: ClassicAction) -> usize{
        self.previous_encounters.iter().filter(|e|{
            e.own_action == action
        }).count()
    }
    pub fn count_actions_other(&self, action: ClassicAction) -> usize{
        self.previous_encounters.iter().filter(|e|{
            e.other_player_action == action
        }).count()
    }
}

impl<ID: UsizeAgentId> Display for OwnHistoryInfoSet<ID> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agent: {}, Rounds: {} \n", self.id, self.previous_encounters.len())?;
        /*let mut s = self.previous_encounters.iter().fold(String::new(), |mut acc, update| {
            acc.push_str(&format!("({:?}){:#}-{:#} #  ", update.side, update.own_action, update.other_player_action));
            acc
        });

         */
        for r in 0..self.previous_encounters.len(){
            let enc = &self.previous_encounters[r];
            write!(f, "\tround: {:3.}, paired against {},\tplayed {}\tagainst {}\n",
                r, ID::make_from_usize( enc.other_id.as_usize()), enc.own_action, enc.other_player_action)?;
        }
        write!(f, "")
    }
}
/*
impl InformationSet<ClassicGameDomainNumbered> for OwnHistoryInfoSetNumbered{
    fn agent_id(&self) -> &AgentNum {
        &self.id
    }

    fn is_action_valid(&self, action: &ClassicAction) -> bool {
        true
    }

    fn update(&mut self, update: ClassicGameUpdate<AgentNum>) -> Result<(), ClassicGameError<AgentNum>> {
        let encounter = update.encounters[self.id as usize];
        self.previous_encounters.push(encounter);
        Ok(())
    }
}

 */

impl<ID: UsizeAgentId> InformationSet<ClassicGameDomain<ID>> for OwnHistoryInfoSet<ID> {
    fn agent_id(&self) -> &ID {
        &self.id
    }

    fn is_action_valid(&self, _action: &ClassicAction) -> bool {
        true
    }

    fn update(&mut self, update: ClassicGameUpdate<ID>) -> Result<(), ClassicGameError<ID>> {

        let report = update.encounters[self.id.as_usize()];
        self.previous_encounters.push(report);
        trace!("After info set update, with {} previous actions", self.previous_encounters.len());
        Ok(())
    }
}

impl<ID: UsizeAgentId> ScoringInformationSet<ClassicGameDomain<ID>> for OwnHistoryInfoSet<ID>{
    type RewardType = i32;

    fn current_subjective_score(&self) -> Self::RewardType {
        self.previous_encounters.iter().map(|r|{
            r.calculate_reward(&self.reward_table)
        }).sum()
    }

    fn penalty_for_illegal(&self) -> Self::RewardType {
        -100
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct OwnHistoryTensorRepr{
    shape: [i64; 2]
}

impl OwnHistoryTensorRepr{
    pub fn new(number_of_rounds: usize) -> Self{
        Self{
            shape: [2, number_of_rounds as i64]
        }
    }
    pub fn shape(&self) -> &[i64]{
        &self.shape[..]
    }
}



impl WayToTensor for OwnHistoryTensorRepr{
    fn desired_shape(&self) -> &[i64] {
        &self.shape[..]
    }
}

pub type OwnHistoryInfoSetNumbered = OwnHistoryInfoSet<AgentNum>;

impl<ID: UsizeAgentId> ConvertToTensor<OwnHistoryTensorRepr> for OwnHistoryInfoSet<ID>{
    fn try_to_tensor(&self, way: &OwnHistoryTensorRepr) -> Result<Tensor, TensorRepresentationError> {
        let max_number_of_actions = way.shape()[1];
        if self.previous_encounters.len() > max_number_of_actions as usize{
            return Err(TensorRepresentationError::InfoSetNotFit {
                info_set: format!("Own encounter history information set with history of length {}", self.previous_encounters.len()),
                shape: Vec::from(way.shape()),
            });
        }
        let mut own_actions: Vec<f32> = self.previous_encounters.iter().map(|e|{
            e.own_action.as_usize() as f32 + 1.0
        }).collect();
        own_actions.resize_with(max_number_of_actions as usize, ||0.0);
        let mut other_actions: Vec<f32> = self.previous_encounters.iter().map(|e|{
            e.other_player_action.as_usize()  as f32 + 1.0
        }).collect();
        other_actions.resize_with(max_number_of_actions as usize, ||0.0);

        let own_tensor = Tensor::f_from_slice(&own_actions[..])?;
        let other_tensor = Tensor::f_from_slice(&other_actions[..])?;

        let result = Tensor::f_stack(&[own_tensor, other_tensor], 0)?
            .flatten(0, -1);
        Ok(result)

    }
}

impl<ID: UsizeAgentId> PresentPossibleActions<ClassicGameDomain<ID>> for OwnHistoryInfoSet<ID>{
    type ActionIteratorType = [ClassicAction;2];

    fn available_actions(&self) -> Self::ActionIteratorType {
        [ClassicAction::Cooperate, ClassicAction::Defect]
    }
}

impl<ID: UsizeAgentId> Renew<()> for OwnHistoryInfoSet<ID>{
    fn renew_from(&mut self, _base: ()) {
        self.previous_encounters.clear()
    }
}
