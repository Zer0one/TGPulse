//! Optional frontend Holder convenience, not a hardware latch or auto-start.
use tgpulse_core::model1::MvdHolderContext;

#[derive(Default)]
pub struct HolderAuto {
    frames: u8,
}

impl HolderAuto {
    /// Press only until the program acknowledges it (at most 60 emulated
    /// frames). Credit/session eligibility and the game's own latch rearm it;
    /// no wall-clock timer, game-memory writes or Trigger synthesis.
    pub fn frame(&mut self, automatic: bool, state: MvdHolderContext) -> bool {
        let eligible = state.in_game || (state.credits > 0 && state.credits != u16::MAX);
        if !automatic || !eligible || state.latched {
            self.frames = 0;
            return false;
        }
        let pressed = self.frames < 60;
        self.frames = self.frames.saturating_add(1).min(60);
        pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn holder_pulses_acknowledge_rearm_and_stop_in_manual_mode() {
        let mut auto = HolderAuto::default();
        let mut state = MvdHolderContext {
            latched: false,
            credits: 0,
            in_game: false,
        };
        assert!(!auto.frame(true, state));
        state.credits = 1;
        assert!(auto.frame(true, state));
        state.latched = true;
        assert!(!auto.frame(true, state));
        // Starting a game clears its latch and consumes the credit.
        state.credits = 0;
        state.in_game = true;
        state.latched = false;
        assert!(auto.frame(true, state));
        assert!(!auto.frame(false, state));
        for _ in 0..60 {
            assert!(auto.frame(true, state));
        }
        assert!(!auto.frame(true, state));
    }
}
