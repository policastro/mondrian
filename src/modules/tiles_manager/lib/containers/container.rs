use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::modules::tiles_manager::lib::tm::error::TilesManagerError;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimationPlayer;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashSet;

type Result<T> = std::result::Result<T, TilesManagerError>;

pub trait Container {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animator: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<()>;
    fn contains(&self, point: (i32, i32)) -> bool;
}

impl Container for WinTree {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animation_player: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<()> {
        let leaves: Vec<AreaLeaf<WindowRef>> = self.leaves(border_pad, Some(ignored_wins));

        for leaf in &leaves {
            if !leaf.id.is_visible() {
                self.remove(leaf.id);
                return self.update(border_pad, tile_pad, animation_player, ignored_wins);
            };
            let area = leaf.viewbox.pad_xy(tile_pad);
            leaf.id.restore(false);
            let borders = leaf.id.get_borders().unwrap_or((0, 0, 0, 0));
            let borders = (
                borders.0.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.1.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.2.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.3.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
            );
            let area = area.shift((-borders.0, -borders.1, borders.2 + borders.0, borders.3 + borders.1));
            animation_player.queue(leaf.id, area, false);
        }
        Ok(())
    }

    fn contains(&self, point: (i32, i32)) -> bool {
        self.get_area().contains(point)
    }
}
