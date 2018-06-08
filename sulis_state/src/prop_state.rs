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

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, AnimationState};
use sulis_module::{Item, LootList, Prop, prop};
use sulis_module::area::PropData;

use entity_state::AreaDrawable;
use {ChangeListenerList, EntityTextureCache, ItemList, ItemState, Location};

pub enum Interactive {
    Not,
    Container {
        items: ItemList,
        loot_to_generate: Option<Rc<LootList>>,
        temporary: bool,
    },
    Door {
        open: bool,
    },
}

pub struct PropState {
    pub prop: Rc<Prop>,
    pub location: Location,
    pub animation_state: AnimationState,
    pub listeners: ChangeListenerList<PropState>,
    interactive: Interactive,

    marked_for_removal: bool,
}

impl fmt::Debug for PropState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prop: {:?}", self.prop)?;
        write!(f, "Location: {:?}", self.location)
    }
}

impl PropState {
    pub(crate) fn new(prop_data: &PropData, location: Location, temporary: bool) -> PropState {
        let mut items = ItemList::new();
        for item in prop_data.items.iter() {
            items.add(ItemState::new(Rc::clone(item)));
        }

        let interactive = match prop_data.prop.interactive {
            prop::Interactive::Not => {
                if !items.is_empty() { warn!("Attempted to add items to a non-container prop"); }
                Interactive::Not
            },
            prop::Interactive::Container { ref loot } => {
                Interactive::Container {
                    items,
                    loot_to_generate: loot.clone(),
                    temporary,
                }
            },
            prop::Interactive::Door { initially_open, .. } => {
                Interactive::Door {
                    open: initially_open
                }
                // TODO set pass and vis if in closed state
            }
        };

        PropState {
            prop: Rc::clone(&prop_data.prop),
            location,
            interactive,
            animation_state: AnimationState::default(),
            listeners: ChangeListenerList::default(),
            marked_for_removal: false,
        }
    }

    pub (crate) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    pub fn might_contain_items(&self) -> bool {
        match self.interactive {
            Interactive::Container { ref items, ref loot_to_generate, .. } => {
                if !items.is_empty() { return true; }
                if loot_to_generate.is_some() { return true; }

                false
            },
            _ => false,
        }
    }

    pub fn is_door(&self) -> bool {
        match self.interactive {
            Interactive::Door { .. } => true,
            _ => false,
        }
    }

    pub fn is_container(&self) -> bool {
        match self.interactive {
            Interactive::Container { .. } => true,
            _ => false,
        }
    }

    pub fn toggle_active(&mut self) {
        self.animation_state.toggle(animation_state::Kind::Active);
        if !self.is_active() { return; }

        match self.interactive {
            Interactive::Not => (),
            Interactive::Container { ref mut items, ref mut loot_to_generate, .. } => {
                let loot = match loot_to_generate.take() {
                    None => return,
                    Some(loot) => loot,
                };

                info!("Generating loot for prop from '{}'", loot.id);
                let generated_items = loot.generate();
                for (qty, item) in generated_items {
                    let item_state = ItemState::new(item);
                    items.add_quantity(qty, item_state);
                }
            },
            Interactive::Door { ref mut open } => {
                let cur_open = *open;
                *open = !cur_open;

                // TODO implement set pass and vis
            },
        }
    }

    pub fn add_item(&mut self, item: ItemState) {
        match self.interactive {
            Interactive::Container { ref mut items, .. } => {
                items.add(item);
            },
            _ => warn!("Attempted to add item to a non-container prop {}", self.prop.id),
        }
        self.listeners.notify(&self);
    }

    pub fn add_items(&mut self, items_to_add: Vec<(u32, Rc<Item>)>) {
        match self.interactive {
            Interactive::Container { ref mut items, .. } => {
                for (qty, item) in items_to_add {
                    let item_state = ItemState::new(item);
                    items.add_quantity(qty, item_state);
                }
            },
            _ => warn!("Attempted to add items to a non-container prop {}", self.prop.id),
        }
        self.listeners.notify(&self);
    }

    pub fn items(&self) -> Option<&ItemList> {
        match self.interactive {
            Interactive::Container { ref items, .. } => Some(&items),
            _ => None,
        }
    }

    pub fn remove_all_at(&mut self, index: usize) -> Option<(u32, ItemState)> {
        let item_state = match self.interactive {
            Interactive::Container { ref mut items, .. } => {
                items.remove_all_at(index)
            },
            _ => {
                warn!("Attempted to remove items from a non-container prop {}", self.prop.id);
                None
            }
        };
        self.notify_and_check();
        item_state
    }

    pub fn remove_one_at(&mut self, index: usize) -> Option<ItemState> {
        let item_state = match self.interactive {
            Interactive::Container { ref mut items, .. } => {
                items.remove(index)
            },
            _ => {
                warn!("Attempted to remove item from a non-container prop {}", self.prop.id);
                None
            }
        };
        self.notify_and_check();
        item_state
    }

    fn notify_and_check(&mut self) {
        self.listeners.notify(&self);
        match self.interactive {
            Interactive::Container { ref items, temporary, .. } => {
                if items.is_empty() && temporary { self.marked_for_removal = true; }
            },
            _ => (),
        }
    }

    pub fn is_active(&self) -> bool {
        self.animation_state.contains(animation_state::Kind::Active)
    }

    pub fn append_to_draw_list(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        self.prop.append_to_draw_list(draw_list, &self.animation_state, x, y, millis);
    }
}

impl AreaDrawable for PropState {
    fn cache(&mut self, _renderer: &mut GraphicsRenderer, _texture_cache: &mut EntityTextureCache) { }

    fn draw(&self, renderer: &mut GraphicsRenderer,
            scale_x: f32, scale_y: f32, x: f32, y: f32, millis: u32, alpha: f32) {
        let x = x + self.location.x as f32;
        let y = y + self.location.y as f32;

        let mut draw_list = DrawList::empty_sprite();
        draw_list.set_scale(scale_x, scale_y);
        draw_list.set_alpha(alpha);
        self.append_to_draw_list(&mut draw_list, x, y, millis);
        renderer.draw(draw_list);
    }

    fn location(&self) -> &Location {
        &self.location
    }
}
