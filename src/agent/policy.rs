use amfi::agent::{AgentIdentifier, Policy};
use crate::agent::OwnHistoryInfoSet;
use crate::domain::ClassicAction::Cooperate;
use crate::domain::{ClassicAction, ClassicGameDomain, UsizeAgentId};

pub struct SwitchAfterTwo{
}

impl<ID: UsizeAgentId> Policy<ClassicGameDomain<ID>> for SwitchAfterTwo{
    type InfoSetType = OwnHistoryInfoSet<ID>;

    fn select_action(&self, state: &Self::InfoSetType) -> Option<ClassicAction> {

        if let Some(last_report) = state.previous_encounters().last(){
            let mut other_action = last_report.other_player_action;
            for i in (0..state.previous_encounters().len()-1).rev(){
                if state.previous_encounters()[i].other_player_action == other_action{
                    return Some(other_action)
                } else {
                    other_action = state.previous_encounters()[i].other_player_action;
                }
            }
            Some(Cooperate)
        } else {
            Some(Cooperate)
        }
    }
}