/// A menu's highlight: ▲ and ▼ move it, wrapping around the ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListMenu {
    highlighted: usize,
    len: usize,
}

impl ListMenu {
    pub fn new(len: usize) -> Self {
        ListMenu {
            highlighted: 0,
            len,
        }
    }

    pub fn highlighted(self) -> usize {
        self.highlighted
    }

    pub fn step(self, down: bool) -> Self {
        let highlighted = if down {
            (self.highlighted + 1) % self.len
        } else {
            (self.highlighted + self.len - 1) % self.len
        };
        ListMenu {
            highlighted,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_wraps_both_ways() {
        let menu = ListMenu::new(3);
        assert_eq!(menu.step(false).highlighted(), 2);
        assert_eq!(menu.step(true).step(true).step(true).highlighted(), 0);
    }
}
