use std::sync::Arc;

use anyhow::anyhow;
use dioxus::prelude::*;

use crate::octopus::meter::MeterManager;
use crate::octopus::property::Property;
use crate::octopus::graphql;

#[derive(Props, Clone)]
pub struct PropertiesPageProps {
    account_id: String,
    meter_manager: Arc<MeterManager>,
    path: Vec<String>,
}

impl PartialEq for PropertiesPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && self.path == other.path
    }
}

#[component]
pub fn PropertiesPage(props: PropertiesPageProps) -> Element {
    // Fetch the list of all properties, this runs once when the component is mounted.
    let properties_resource = use_resource({
        let meter_manager = props.meter_manager.clone();
        let account_id = props.account_id.clone();
        move || {
            let meter_manager = meter_manager.clone();
            let account_id = account_id.clone();
            async move { meter_manager.get_properties(&account_id).await }
        }
    });

    match &*properties_resource.read() {
        None => rsx! {
            div { "Loading Properties for account {props.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load properties: {error:?}"))?,
        Some(Ok(property_list)) => {

            // Are we looking at one bill?
            if let Some(property_id) = props.path.first() {
                if let Some(property) = property_list.property_map.get(property_id) {
                    rsx! {
                        PropertyDetail {
                            key: "{property_id}",
                            account_id: props.account_id.clone(),
                            property: property.clone(),
                            meter_manager: props.meter_manager.clone(),
                        }
                    }
                }
                else {
                    rsx! {
                        div { "No such property {property_id}" }
                    }
                }
            }
            else {
                rsx! {
                    table {
                        {Property::gui_summary_header()?}
                        for property in property_list.property_map.values() {
                            {property.gui_summary_line()?}
                        }
                    }
                }
            }
        },
    }
}

#[derive(Props, Clone)]
struct PropertyDetailProps {
    property: Arc<graphql::meter::account_properties_meters::PropertyType>,
    account_id: String,
    meter_manager: Arc<MeterManager>,
}

impl PartialEq for PropertyDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.property, &other.property)
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
    }
}

#[component]
fn PropertyDetail(props: PropertyDetailProps) -> Element {

    rsx! {
        {props.property.gui_display()?}
    }
}
