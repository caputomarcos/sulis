//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::fmt;
use std::rc::Rc;

use sulis_core::io::DrawList;
use sulis_core::ui::{animation_state, AnimationState};
use sulis_module::Prop;
use {ChangeListenerList, ItemState, Location};

pub struct PropState {
    pub prop: Rc<Prop>,
    pub location: Location,
    pub animation_state: AnimationState,
    pub items: Vec<ItemState>,
    pub listeners: ChangeListenerList<PropState>,
}

impl fmt::Debug for PropState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prop: {:?}", self.prop)?;
        write!(f, "Location: {:?}", self.location)
    }
}

impl PropState {
    pub(crate) fn new(prop: Rc<Prop>, location: Location) -> PropState {
        let mut items = Vec::new();
        for item in prop.items.iter() {
            items.push(ItemState::new(Rc::clone(item)));
        }

        PropState {
            prop,
            location,
            animation_state: AnimationState::default(),
            items,
            listeners: ChangeListenerList::default(),
        }
    }

    pub fn toggle_active(&mut self) {
        let kind = animation_state::Kind::Active;

        self.animation_state.toggle(kind);
    }

    pub fn is_active(&self) -> bool {
        self.animation_state.contains(animation_state::Kind::Active)
    }

    /// Removes an item from this Prop at the specified index and returns it
    pub fn remove_item(&mut self, index: usize) -> ItemState {
        let item = self.items.remove(index);
        self.listeners.notify(&self);
        item
    }

    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    pub fn append_to_draw_list(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        self.prop.append_to_draw_list(draw_list, &self.animation_state, x, y, millis);
    }
}
