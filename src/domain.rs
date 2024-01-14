use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use amfi_core::agent::{AgentIdentifier};
use amfi_core::error::{AmfiError, ConvertError};
use amfi_core::domain::{Action, DomainParameters, Reward};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use tch::Tensor;
use amfi_rl::tensor_data::ActionTensor;
use crate::domain::TwoPlayersStdName::{Alice, Bob};
use crate::env::PairingVec;
use crate::{AsymmetricRewardTable, Side};
use crate::domain::ClassicAction::{Down, Up};

pub trait AsUsize: Serialize{
    fn as_usize(&self) -> usize;
    fn make_from_usize(u: usize) -> Self;
}
pub type AgentNum = u32;

impl AsUsize for AgentNum{
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn make_from_usize(u: usize) -> Self {
        u as AgentNum
    }
}
/*
impl<T: Enum + Copy> AsUsize for T{
    fn as_usize(&self) -> usize {
        self.into_usize()
    }
}

 */
pub trait UsizeAgentId: AgentIdentifier + AsUsize + Copy + Serialize{}
impl<T: AsUsize + AgentIdentifier + Copy + Serialize> UsizeAgentId for T{

}

/// Choice from two possible actions in simple classic game.
/// In different problems and different papers they are differently called.
/// In prisoners' dilemma the can be referenced as as _Defect_ and _Cooperate_.
/// In chicken game they are commonly named _Hawk_ and _Dove_.
/// Often one can be described as _Aggressive_ and other as _Passive_, (or _Selfish_ and _Caring_).
/// In some games however such description does not make sense, for example in
/// [battle of sexes](https://en.wikipedia.org/wiki/Battle_of_the_sexes_(game_theory)).
/// This semantic inconsistency problem will be better addressed in the future, for now
/// it is advised to make note which variant represents which action.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum ClassicAction {
    Up,
    Down
}

impl ClassicAction{
    /// Represent variants in prisoner game (Up -> Defect, Down -> Cooperate)
    pub fn str_prisoner(&self) -> &'static str{
        match self {
            Up => "Defect",
            Down => "Cooperate"
        }
    }
    /// Represent variants in chicken game (Up -> Hawk, Down -> Down)
    pub fn str_chicken(&self) -> &'static str{
        match self {
            Up => "Hawk",
            Down => "Dove"
        }
    }
    /// Represent variants in battle of sexes game (Up -> Fight, Down -> Ballet)
    pub fn str_sexes(&self) -> &'static str{
        match self {
            Up => "Hawk",
            Down => "Dove"
        }
    }
}
#[allow(non_upper_case_globals)]
/// Alias for prisoner's defect
pub const Defect: ClassicAction = Up;


//#[allow(non_upper_case_globals)]
//pub const Aggressive: ClassicAction = Up;
/// Alias for kindness definition of action Up
#[allow(non_upper_case_globals)]
pub const Selfish: ClassicAction = Up;
/// Alias for battle of sexes Up action
#[allow(non_upper_case_globals)]
pub const Fight: ClassicAction = Up;


/// Alias for prisoner's cooperate
#[allow(non_upper_case_globals)]
pub const Cooperate: ClassicAction = Down;
//#[allow(non_upper_case_globals)]
//pub const Passive: ClassicAction = Down;
/// Alias for kindness definition of action Down
#[allow(non_upper_case_globals)]
pub const Caring: ClassicAction = Down;
/// Alias for battle of sexes Down action
#[allow(non_upper_case_globals)]
pub const Ballet: ClassicAction = Down;



impl AsUsize for ClassicAction{
    fn as_usize(&self) -> usize {
        self.into_usize()
    }

    fn make_from_usize(u: usize) -> Self {
        ClassicAction::from_usize(u)
    }
}

impl Display for ClassicAction {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate(){
            match self{
                ClassicAction::Up => write!(f, "B"),
                ClassicAction::Down => write!(f, "C")
            }
        } else{
            write!(f, "{:?}", self)
        }

    }
}


impl Action for ClassicAction {}
//--------------------------------------
impl ActionTensor for ClassicAction {
    fn to_tensor(&self) -> Tensor {
        match self{
            Up => Tensor::from_slice(&[0.0f32;1]),
            Down => Tensor::from_slice(&[1.0f32;1])
        }
    }


    /// ```
    /// use amfi_classic::domain::ClassicAction;
    /// use amfi_classic::domain::ClassicAction::Down;
    /// use tch::Tensor;
    /// use amfi_rl::tensor_data::ActionTensor;
    /// let t = Tensor::from_slice(&[1i64;1]);
    /// let action = ClassicAction::try_from_tensor(&t).unwrap();
    /// assert_eq!(action, Down);
    /// ```
    fn try_from_tensor(t: &Tensor) -> Result<Self, ConvertError> {


        let v: Vec<i64> = match Vec::try_from(t){
            Ok(v) => v,
            Err(_) =>{
                return Err(ConvertError::ActionDeserialize(format!("{}", t)))
            }
        };
        match v[0]{
            0 => Ok(Defect),
            1 => Ok(Down),
            _ => Err(ConvertError::ActionDeserialize(format!("{}", t)))
        }
    }
}


#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum ClassicGameError<ID: AgentIdentifier> {
    #[error("Performed different action (chosen: {chosen:?}, logged: {logged:?})")]
    DifferentActionPerformed{
        chosen: ClassicAction,
        logged: ClassicAction
    },
    #[error("Order in game was violated. Current player given by current_player(): {expected:?} given: {acted:}")]
    GameViolatedOrder{
        acted: ID,
        expected: Option<ID>
    },
    #[error("Environment logged action {0}, but none was performed")]
    NoLastAction(ClassicAction),
    #[error("Player: {0} played after GameOver")]
    ActionAfterGameOver(ID),
    #[error("Player: {0} played out of order")]
    ActionOutOfOrder(ID),
    #[error("Value can't be probability: {0}")]
    NotAProbability(f64),
    #[error("Odd number of players: {0}")]
    ExpectedEvenNumberOfPlayers(u32),
    #[error("Update does no include requested encounter report for agent: {0}")]
    EncounterNotReported(AgentNum),
}

/*
impl Into<AmfiError<PrisonerDomain>> for PrisonerError {
    fn into(self) -> AmfiError<PrisonerDomain> {
        AmfiError::Game(self)
    }
}

 */
impl<ID: UsizeAgentId> From<ClassicGameError<ID>> for AmfiError<ClassicGameDomain<ID>>{
    fn from(value: ClassicGameError<ID>) -> Self {
        AmfiError::Game(value)
    }
}


#[derive(Clone, Debug, Serialize)]
pub struct ClassicGameDomain<ID: AgentIdentifier>{
    _id: PhantomData<ID>
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct EncounterReport<ID: UsizeAgentId> {

    pub own_action: ClassicAction,
    pub other_player_action: ClassicAction,
    pub side: Side,
    pub other_id: ID,

}


impl<ID: UsizeAgentId> EncounterReport<ID>{
    pub fn left_action(&self) -> ClassicAction{
        match self.side{
            Side::Left => self.own_action,
            Side::Right => self.other_player_action
        }
    }
    pub fn right_action(&self) -> ClassicAction{
        match self.side{
            Side::Left => self.other_player_action,
            Side::Right => self.own_action
        }
    }
    pub fn side_action(&self, side: Side) -> ClassicAction{
        match side{
            Side::Left => self.left_action(),
            Side::Right => self.right_action(),
        }
    }
    pub fn own_side(&self) -> Side{
        self.side
    }
    pub fn calculate_reward<R: Reward + Copy>(&self, table: &AsymmetricRewardTable<R>) -> R{
        let (left, right) = match self.side{
            Side::Left => (self.own_action, self.other_player_action),
            Side::Right => (self.other_player_action, self.own_action),
        };
        table.reward_for_side(self.side, left, right)
    }
}

pub type EncounterReportNamed = EncounterReport<TwoPlayersStdName>;
pub type EncounterReportNumbered = EncounterReport<AgentNum>;

impl<ID: UsizeAgentId> Display for EncounterReport<ID> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Update [own action: {}, opponent's action: {}]", self.own_action, self.other_player_action)
    }
}

//impl StateUpdate for PrisonerUpdate{}

//pub type PrisonerId = u8;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Enum, Serialize)]
pub enum TwoPlayersStdName {
    Alice,
    Bob,

}



impl AsUsize for TwoPlayersStdName {
    fn as_usize(&self) -> usize {
        self.into_usize()
    }

    fn make_from_usize(u: usize) -> Self {
        TwoPlayersStdName::from_usize(u)
    }
}


impl TwoPlayersStdName {
    pub fn other(self) -> Self{
        match self{
            Self::Alice => Bob,
            Self::Bob => Alice
        }
    }
}



impl AgentIdentifier for TwoPlayersStdName {}

#[derive(Debug, Copy, Clone, Default)]
pub struct PrisonerMap<T>{
    alice_s: T,
    bob_s: T
}
impl<T> Display for PrisonerMap<T> where T: Display{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Alice: {} | Bob: {}]", self[Alice], self[Bob])
    }
}

impl Display for TwoPlayersStdName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> PrisonerMap<T>{
    pub fn new(alice_s: T, bob_s: T) -> Self{
        Self{ alice_s, bob_s }
    }

}

impl<T> Index<TwoPlayersStdName> for PrisonerMap<T>{
    type Output = T;

    fn index(&self, index: TwoPlayersStdName) -> &Self::Output {
        match index{
            TwoPlayersStdName::Bob => &self.bob_s,
            TwoPlayersStdName::Alice => &self.alice_s
        }
    }
}

impl<T> IndexMut<TwoPlayersStdName> for PrisonerMap<T>{

    fn index_mut(&mut self, index: TwoPlayersStdName) -> &mut Self::Output {
        match index{
            TwoPlayersStdName::Bob => &mut self.bob_s,
            TwoPlayersStdName::Alice => &mut self.alice_s
        }
    }
}


pub const TWO_PLAYERS_STD_NAMED:[TwoPlayersStdName;2] = [TwoPlayersStdName::Alice, TwoPlayersStdName::Bob];

pub type IntReward = i64;


#[derive(Debug, Clone, Serialize)]
pub struct ClassicGameUpdate<ID: UsizeAgentId>{
    pub encounters: Arc<Vec<EncounterReport<ID>>>,
    pub pairing:  Option<Arc<PairingVec<ID>>>
}

impl<ID: UsizeAgentId> DomainParameters for ClassicGameDomain<ID> {
    type ActionType = ClassicAction;
    type GameErrorType = ClassicGameError<ID>;
    type UpdateType = ClassicGameUpdate<ID>;
    type AgentId = ID;
    type UniversalReward = IntReward;
}
pub type ClassicGameDomainNamed = ClassicGameDomain<TwoPlayersStdName>;
pub type ClassicGameDomainNumbered = ClassicGameDomain<AgentNum>;
