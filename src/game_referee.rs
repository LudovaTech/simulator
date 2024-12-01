use nalgebra::Vector2;

use crate::infos;
use crate::simulator::FieldWallKind;

#[derive(PartialEq)]
pub enum RefereeAction {
    ContinueMatch,
    NewRound,
}


/// Stocke les données de la partie mais ne modifie pas directement l'action de la partie
#[derive(Default)]
pub struct GameReferee {
    pub score_team_left: u32,
    pub score_team_right: u32,
}

impl GameReferee {
    pub fn maybe_goal(&mut self, position_of_ball: &Vector2<f32>, wall: &FieldWallKind) -> RefereeAction {
        // On considére que le fait que la balle a touché le mur du fond
        // a déjà été vérifié (dans process_collisions)
        // On ne vérifie donc que la position y de la balle
        if (((infos::FIELD_WIDTH / 2.0) - (infos::GOAL_WIDTH / 2.0))
            ..(infos::FIELD_WIDTH / 2.0) + (infos::GOAL_WIDTH / 2.0))
            .contains(&position_of_ball.y)
        {
            // GOAL
            match wall {
                FieldWallKind::Left => self.score_team_left += 1,
                FieldWallKind::Right => self.score_team_right += 1,
                _ => panic!("maybe_goal called with unexpected values"),
            }
            println!("GOAL");
            return RefereeAction::NewRound;
        }
        return RefereeAction::ContinueMatch;
    }
}
