/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use core::cell::Cell;

use rs_matter_macros::idl_import;

use strum::{EnumDiscriminants, FromRepr};

use crate::error::Error;
use crate::tlv::TLVElement;
use crate::transport::exchange::Exchange;
use crate::attribute_enum;

use super::objects::*;

idl_import!(clusters = ["BooleanState"]);

pub use boolean_state::ID;

#[derive(FromRepr, EnumDiscriminants)]
#[repr(u16)]
pub enum Attributes {
    State(AttrType<bool>) = 0x0,
}

attribute_enum!(Attributes);

pub const CLUSTER: Cluster<'static> = Cluster {
    id: ID as _,
    feature_map: 0,
    attributes: &[
        FEATURE_MAP,
        ATTRIBUTE_LIST,
        Attribute::new(
            AttributesDiscriminants::State as u16,
            Access::RV,
            Quality::SN,
        ),
    ],
    commands: &[],
};

#[derive(Clone)]
pub struct ContactSensorCluster {
    data_ver: Dataver,
    state: Cell<bool>,
}

impl ContactSensorCluster {
    pub const fn new(data_ver: Dataver) -> Self {
        Self {
            data_ver,
            state: Cell::new(false),
        }
    }

    pub fn get(&self) -> bool {
        self.state.get()
    }

    pub fn set(&self, on: bool) {
        if self.state.get() != on {
            self.state.set(on);
            self.data_ver.changed();
        }
    }

    pub fn read(
        &self,
        _exchange: &Exchange,
        attr: &AttrDetails,
        encoder: AttrDataEncoder,
    ) -> Result<(), Error> {
        if let Some(writer) = encoder.with_dataver(self.data_ver.get())? {
            if attr.is_system() {
                CLUSTER.read(attr.attr_id, writer)
            } else {
                match attr.attr_id.try_into()? {
                    Attributes::State(codec) => codec.encode(writer, self.state.get()),
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn write(
        &self,
        _exchange: &Exchange,
        attr: &AttrDetails,
        data: AttrData,
    ) -> Result<(), Error> {
        let data = data.with_dataver(self.data_ver.get())?;

        match attr.attr_id.try_into()? {
            Attributes::State(codec) => self.set(codec.decode(data)?),
        }

        self.data_ver.changed();

        Ok(())
    }

    pub fn invoke(
        &self,
        _exchange: &Exchange,
        _cmd: &CmdDetails,
        _data: &TLVElement,
        _encoder: CmdDataEncoder,
    ) -> Result<(), Error> {
        self.data_ver.changed();

        Ok(())
    }
}

impl Handler for ContactSensorCluster {
    fn read(
        &self,
        exchange: &Exchange,
        attr: &AttrDetails,
        encoder: AttrDataEncoder,
    ) -> Result<(), Error> {
        ContactSensorCluster::read(self, exchange, attr, encoder)
    }

    fn write(&self, exchange: &Exchange, attr: &AttrDetails, data: AttrData) -> Result<(), Error> {
        ContactSensorCluster::write(self, exchange, attr, data)
    }

    fn invoke(
        &self,
        exchange: &Exchange,
        cmd: &CmdDetails,
        data: &TLVElement,
        encoder: CmdDataEncoder,
    ) -> Result<(), Error> {
        ContactSensorCluster::invoke(self, exchange, cmd, data, encoder)
    }
}

// TODO: Might be removed once the `on` member is externalized
impl NonBlockingHandler for ContactSensorCluster {}

impl ChangeNotifier<()> for ContactSensorCluster {
    fn consume_change(&mut self) -> Option<()> {
        self.data_ver.consume_change(())
    }
}
