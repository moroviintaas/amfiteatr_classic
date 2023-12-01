use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Index, IndexMut, Sub};
use enum_map::{enum_map, EnumMap};
use amfi::domain::Reward;
use crate::domain::{ClassicAction, IntReward};
use crate::domain::ClassicAction::{Cooperate, Defect};


pub type Level1ActionMap<T> = EnumMap<ClassicAction, T>;
pub type Level2ActionMap<T> = EnumMap<ClassicAction, Level1ActionMap<T>>;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ActionCounter<T: Copy + Clone + Debug + PartialEq>(Level2ActionMap<T>);

impl<T: Copy + Clone + Debug + PartialEq> ActionCounter<T>{
    pub fn new(map: Level2ActionMap<T>) -> Self{
        Self(map)
    }
}

impl ActionCounter<i64>{
    pub fn zero() -> Self{
        Self::default()
    }
}

impl<T: Copy + Clone + Debug + PartialEq> Index<ClassicAction> for ActionCounter<T>{
    type Output = Level1ActionMap<T>;

    fn index(&self, index: ClassicAction) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Copy + Clone + Debug + PartialEq> IndexMut<ClassicAction> for ActionCounter<T>{

    fn index_mut(&mut self, index: ClassicAction) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<'a, T: Copy + Clone + Debug + PartialEq + Default> Default for ActionCounter<T>{
    fn default() -> Self {
        Self(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect => T::default(),
                ClassicAction::Cooperate => T::default()
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect => T::default(),
                ClassicAction::Cooperate => T::default()
            }
        })
    }
}

impl<T: Copy + Clone + Debug + Add<Output = T> + PartialEq> Add for ActionCounter<T>{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ActionCounter::new(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Defect][ClassicAction::Defect]
                    + rhs[ClassicAction::Defect][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Defect][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Defect][ClassicAction::Cooperate],
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Cooperate][ClassicAction::Defect]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Cooperate][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Cooperate],
            },
        })
    }
}

impl<'a, T: Copy + Clone + Debug + Add<Output = T> + PartialEq> Add<&'a Self> for ActionCounter<T>{
    type Output = Self;

    fn add(self, rhs: &'a Self) -> Self::Output {
        ActionCounter(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Defect][ClassicAction::Defect]
                    + rhs[ClassicAction::Defect][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Defect][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Defect][ClassicAction::Cooperate],
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Cooperate][ClassicAction::Defect]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Cooperate][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Cooperate],
            },
        })
    }
}

impl<'a, T: Copy + Clone + Debug + AddAssign + PartialEq> AddAssign<&'a Self> for ActionCounter<T>{

    fn add_assign(&mut self, rhs: &'a Self){
        self[Cooperate][Cooperate] += rhs[Cooperate][Cooperate];
        self[Cooperate][Defect] += rhs[Cooperate][Defect];
        self[Defect][Cooperate] += rhs[Defect][Cooperate];
        self[Defect][Defect] += rhs[Defect][Defect];

        /*
        ActionCounter(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Defect][ClassicAction::Defect]
                    + rhs[ClassicAction::Defect][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Defect][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Defect][ClassicAction::Cooperate],
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Cooperate][ClassicAction::Defect]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Cooperate][ClassicAction::Cooperate]
                    + rhs[ClassicAction::Cooperate][ClassicAction::Cooperate],
            },
        })

         */
    }
}

impl<T: Copy + Clone + Debug + Sub<Output = T> + PartialEq> Sub for ActionCounter<T>{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ActionCounter::new(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Defect][ClassicAction::Defect]
                    - rhs[ClassicAction::Defect][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Defect][ClassicAction::Cooperate]
                    - rhs[ClassicAction::Defect][ClassicAction::Cooperate],
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Cooperate][ClassicAction::Defect]
                    - rhs[ClassicAction::Cooperate][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Cooperate][ClassicAction::Cooperate]
                    - rhs[ClassicAction::Cooperate][ClassicAction::Cooperate],
            },
        })
    }
}

impl<'a, T: Copy + Clone + Debug + Sub<Output = T> + PartialEq> Sub<&'a Self> for ActionCounter<T>{
    type Output = Self;

    fn sub(self, rhs: &'a Self) -> Self::Output {
        ActionCounter(enum_map! {
            ClassicAction::Defect => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Defect][ClassicAction::Defect]
                    - rhs[ClassicAction::Defect][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Defect][ClassicAction::Cooperate]
                    - rhs[ClassicAction::Defect][ClassicAction::Cooperate],
            },
            ClassicAction::Cooperate => enum_map! {
                ClassicAction::Defect =>  self[ClassicAction::Cooperate][ClassicAction::Defect]
                    - rhs[ClassicAction::Cooperate][ClassicAction::Defect],
                ClassicAction::Cooperate =>  self[ClassicAction::Cooperate][ClassicAction::Cooperate]
                    - rhs[ClassicAction::Cooperate][ClassicAction::Cooperate],
            },
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct VerboseReward<R: Reward + Copy>{
    table_payoff: R,
    //count_coop_vs_coop: i64,
    //count_coop_vs_defect: i64,
    //count_defect_vs_coop: i64,
    //count_defect_vs_defect: i64,
    action_counts: ActionCounter<i64>

}


impl<R: Reward + Copy> PartialOrd for VerboseReward<R> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.table_payoff.partial_cmp(&other.table_payoff)
    }
}

impl<R: Reward + Copy> VerboseReward<R>{

    pub fn new(table_payoff: R, action_counts: ActionCounter<i64>) -> Self{
        Self{table_payoff, action_counts}
    }

    pub fn with_only_table_payoff(payoff: R) -> Self{
        Self{
            table_payoff: payoff,
            action_counts: ActionCounter::zero()
        }
    }
}

impl VerboseReward<IntReward>{

    pub fn table_payoff(&self) -> IntReward{
        self.table_payoff
    }

    pub fn count_own_actions(&self, action: ClassicAction) -> IntReward{
        match action{
            Defect => self.action_counts[Defect][Cooperate] + self.action_counts[Defect][Defect],
            Cooperate => self.action_counts[Cooperate][Cooperate] + self.action_counts[Cooperate][Defect],
        }
    }

    pub fn count_other_actions(&self, action: ClassicAction) -> IntReward{
        match action{
            Defect => self.action_counts[Cooperate][Defect] + self.action_counts[Defect][Defect],
            Cooperate => self.action_counts[Cooperate][Cooperate] + self.action_counts[Defect][Cooperate],
        }
    }

    pub fn other_coop_as_reward(&self) -> IntReward{
        self.action_counts[Cooperate][Cooperate] + self.action_counts[Defect][Cooperate]
    }

    pub fn f_combine_table_with_other_coop(&self, action_count_weight: f32) -> f32{
        self.table_payoff as f32 + (action_count_weight * self.count_other_actions(Cooperate) as f32)
    }



}



impl<'a, R: Reward + Copy> Add<&'a Self> for VerboseReward<R> {
    type Output = VerboseReward<R>;

    fn add(self, rhs: &'a Self) -> Self::Output {
        Self{
            table_payoff: self.table_payoff + rhs.table_payoff,
            /*
            count_coop_vs_coop: self.count_coop_vs_coop + rhs.count_coop_vs_coop,
            count_coop_vs_defect: self.count_coop_vs_defect + rhs.count_coop_vs_defect,
            count_defect_vs_coop: self.count_defect_vs_coop + rhs.count_defect_vs_coop,
            count_defect_vs_defect: self.count_defect_vs_defect + rhs.count_defect_vs_defect

             */
            action_counts: self.action_counts + rhs.action_counts
        }
    }
}

impl<R: Reward + Copy> Add for VerboseReward<R> {
    type Output = VerboseReward<R>;

    fn add(self, rhs: Self) -> Self::Output {
        Self{
            table_payoff: self.table_payoff + rhs.table_payoff,
            action_counts: self.action_counts + rhs.action_counts,
            /*
            count_coop_vs_coop: self.count_coop_vs_coop + rhs.count_coop_vs_coop,
            count_coop_vs_defect: self.count_coop_vs_defect + rhs.count_coop_vs_defect,
            count_defect_vs_coop: self.count_defect_vs_coop + rhs.count_defect_vs_coop,
            count_defect_vs_defect: self.count_defect_vs_defect + rhs.count_defect_vs_defect

             */
        }
    }
}



impl<'a, R: Reward + Copy> AddAssign<&'a Self> for VerboseReward<R> {
    fn add_assign(&mut self, rhs: &'a Self) {
        self.table_payoff += &rhs.table_payoff;
        /*
        self.count_coop_vs_coop += &rhs.count_coop_vs_coop;
        self.count_coop_vs_defect += &rhs.count_coop_vs_defect;
        self.count_defect_vs_coop += &rhs.count_defect_vs_coop;
        self.count_defect_vs_defect += &rhs.count_defect_vs_defect;

         */

        self.action_counts += &rhs.action_counts;
    }
}

impl<R: Reward + Copy> Sub for VerboseReward<R> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{
            table_payoff: self.table_payoff - rhs.table_payoff,
            /*
            count_coop_vs_coop: self.count_coop_vs_coop - rhs.count_coop_vs_coop,
            count_coop_vs_defect: self.count_coop_vs_defect - rhs.count_coop_vs_defect,
            count_defect_vs_coop: self.count_defect_vs_coop - rhs.count_defect_vs_coop,
            count_defect_vs_defect: self.count_defect_vs_defect - rhs.count_defect_vs_defect

             */
            action_counts : self.action_counts - rhs.action_counts
        }
    }
}

impl<'a, R: Reward + Copy> Sub<&'a Self> for VerboseReward<R> {
    type Output = Self;

    fn sub(self, rhs: &'a Self) -> Self::Output {
        Self{
            table_payoff: self.table_payoff - rhs.table_payoff,
            action_counts : self.action_counts - rhs.action_counts
        }
    }
}





impl<R: Reward + Copy> Reward for VerboseReward<R>{
    fn neutral() -> Self {
        Self{
            table_payoff: R::neutral(),
            action_counts: ActionCounter::zero()
        }
    }
}

