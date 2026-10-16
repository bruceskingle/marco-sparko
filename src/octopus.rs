

pub mod token;
pub mod decimal;
mod account;
mod property;
mod bill;
mod meter;

use std::sync::Arc;

use anyhow::anyhow;
use account::AccountManager;
use async_trait::async_trait;

use dioxus::prelude::*;

use bill::BillManager;
use meter::MeterManager;
use serde::{Deserialize, Serialize};

use time_tz::{Tz, timezones};
use token::{OctopusTokenManager};
use clap::Parser;

use sparko_graphql::TokenManager;
use sparko_graphql::types::{Date, DateTime};
use dioxus_free_icons::{Icon, icons::ld_icons::LdCalendar};
use time_tz::{OffsetDateTimeExt, PrimitiveDateTimeExt};
use crate::{CacheConfig, InitRequested, MarcoSparkoContext, Module, ModuleFactory, ModuleRegistration, PageInfo, octopus::{bill::{AbstractBill, BillDataSet}, property::Property, token::OctopusAuthenticator}};

// include!("octopus/graphql.rs");
include!(concat!(env!("OUT_DIR"), "/graphql.rs"));
include!(concat!(env!("OUT_DIR"), "/crate_info.rs"));

#[cfg(graphql_generation_error)]
compile_error!("graphql built with Errors");


pub type RequestManager = sparko_graphql::AuthenticatedRequestManager<OctopusTokenManager>;


/*
 * It is important that the names of all args here begin with octopus_
 * as only args beginning with octopus- on the command line are passed through to the module.
 * 
 * Underscores in the names of fields on this struct are translated into hyphens on the command line automatically by clap.
 */
#[derive(Parser, Debug, Clone, PartialEq)]
pub struct OctopusArgs {
    /// The Octopus API_KEY to use
    #[arg(short, long, env)]
    octopus_api_key: Option<String>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub api_key:  Option<String>,
    pub billing_timezone: Option<String>,
    #[serde(skip)]
    // #[serde(default = false)]
    pub init: bool,
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            api_key: None,
            billing_timezone: Some("Europe/London".to_string()),
            init: true,
        }
    }
}

pub struct OctopusModule{
    octopus_args: OctopusArgs,
    account_id: String,
    bill_manager: Arc<BillManager>,
    meter_manager: Arc<MeterManager>,
    account_manager: AccountManager,
    billing_timezone: &'static time_tz::Tz,
}

const MODULE_ID: &str = "octopus";


#[derive(Deserialize, Debug, Clone)]
struct LoginForm {
    email: Option<String>,
    password: Option<String>,
    api_key: Option<String>,
    login_method: String,
}

impl OctopusModule {
    async fn new(octopus_args: OctopusArgs, cache_config: Arc<CacheConfig>, profile: Profile, 
        request_manager: Arc<RequestManager>, _verbose: bool) -> anyhow::Result<OctopusModule> {   

        let billing_timezone = Self::get_billing_timezone(&profile);
        let account_manager = AccountManager::new(&cache_config, &request_manager).await?;
        let meter_manager = Arc::new(MeterManager::new(&cache_config, &request_manager));
        let bill_manager = Arc::new(BillManager::new(&cache_config, &request_manager, &meter_manager));

        Ok(OctopusModule {
            octopus_args,
            account_id: account_manager.get_default_account_id().to_string(),
            account_manager,
            bill_manager,
            meter_manager,
            billing_timezone,
        })
    }

    fn get_billing_timezone(profile: &Profile) -> &'static time_tz::Tz {
        // if let Some(profile) = profile {
            if let Some(name) = &profile.billing_timezone {
                if let Some(tz) =  timezones::get_by_name(&name) {
                    return tz;
                }
                panic!("Unable to load billing_timezone '{}'", name);
            }
        // }
        return timezones::db::europe::LONDON;
    }

    pub fn registration() -> ModuleRegistration {
        ModuleRegistration {
            module_id: MODULE_ID.to_string(),
            constructor: Arc::new(OctopusModule::constructor),
        }
    }
    
    pub fn constructor(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>) -> anyhow::Result<Arc<dyn ModuleFactory>> {

            let builder =OctopusModule::builder(context, json_profile)?;

            Ok(Arc::new(builder.build()?))
    }

    pub fn builder(context: Arc<MarcoSparkoContext>, 
        json_profile: Option<serde_json::Value>
    ) -> anyhow::Result<OctopusModuleFactoryBuilder> {

        OctopusModuleFactoryBuilder::new(context, json_profile)
    }
}



fn find_bill<'a>(bill_id: &String, bills: &'a BillDataSet) -> Option<&'a AbstractBill> {
    // bills.data_set.data.get(bill_id)
    if let Some((_attributes, bill)) = bills.data_set.data.get(bill_id) {
        Some(bill)
    }
    else {
        None
    }
}

#[async_trait]
impl Module for OctopusModule {
    fn cli_debug(&self) -> Result<(), anyhow::Error> {
        println!("Args: {:?}", self.octopus_args);
        println!("Octopus Module: account_id={}", self.account_id);
        Ok(())
    }

    fn module_id(&self) -> &'static str {
        MODULE_ID
    }

    fn get_page_list(&self) -> Vec<PageInfo> {
        vec!(
            PageInfo {
                label: "User", 
                path: "user",
            },
            PageInfo {
                label: "Account", 
                path: "account",
            },
            PageInfo {
                label: "Bills",
                path: "bills",
            },
            PageInfo {
                label: "Properties",
                path: "properties",
            },
            PageInfo {
                label: "Meters",
                path: "meters",
            }
        )
    }

    fn get_component(&self, page_id: &str, path: Vec<String>) -> Element {
        match page_id {
            "user" => {
                // let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
                let date_format = time::format_description::parse("[year]-[month]-[day]").unwrap();
                let account_user = &self.account_manager.data_set.data.viewer_;
                let dob = if let Some(date) = &account_user.date_of_birth_ {
                    date.format(&date_format).unwrap()
                }
                else {
                    "".to_string()
                };

                        // tr {
                        //    th { class: "row-header","accounts"} td{"{account_user.accounts_}"}
                        // }
                rsx! {
                    h1 { "Viewer (Current User)" }
                    table { class: "display",
                        tr {
                            th { class: "row-header", "ID" }
                            td { "{account_user.id_}" }
                        }
                        tr {
                            th { class: "row-header", "Full Name" }
                            td { "{account_user.full_name_}" }
                        }
                        tr {
                            th { class: "row-header", "Title" }
                            td { "{account_user.title_}" }
                        }
                        tr {
                            th { class: "row-header", "Preferred Name" }
                            td { "{account_user.preferred_name_}" }
                        }
                        tr {
                            th { class: "row-header", "Given Name" }
                            td { "{account_user.given_name_}" }
                        }
                        tr {
                            th { class: "row-header", "Family Name" }
                            td { "{account_user.family_name_}" }
                        }
                        tr {
                            th { class: "row-header", "Pronouns" }
                            td { {account_user.pronouns_.as_deref().unwrap_or("")} }
                        }
                        tr {
                            th { class: "row-header", "Email" }
                            td { "{account_user.email_}" }
                        }

                        tr {
                            th { class: "row-header", "Mobile" }
                            td { "{account_user.mobile_}" }
                        }
                        tr {
                            th { class: "row-header", "Landline" }
                            td { "{account_user.landline_}" }
                        }
                        tr {
                            th { class: "row-header", "API Key" }
                            td { {account_user.live_secret_key_.as_deref().unwrap_or("")} }
                        }
                        tr {
                            th { class: "row-header", "Is Deceased" }
                            td { "{account_user.is_deceased_}" }
                        }
                        tr {
                            th { class: "row-header", "Date of Birth" }
                            td { {dob} }
                        }
                        tr {
                            th { class: "row-header", "Alternative Phone Numbers" }
                            td { {format!("{:?}", account_user.alternative_phone_numbers_)} }
                        }
                        tr {
                            th { class: "row-header", "Has Family Issues" }
                            td { "{account_user.has_family_issues_}" }
                        }
                        tr {
                            th { class: "row-header", "Is in Hardship" }
                            td { "{account_user.is_in_hardship_}" }
                        }
                        tr {
                            th { class: "row-header", "Is opted in to Wheel of Fortune" }
                            td { "{account_user.is_opted_in_to_wof_}" }
                        }
                    }

                    h2 { "Accounts" }

                    for account in &account_user.accounts_ {
                        div {
                            h3 { "Account {account.number_}" }
                            table { class: "display",
                                tr {
                                    th { class: "row-header", "Brand" }
                                    td { "{account.brand_}" }
                                }
                                tr {
                                    th { class: "row-header", "Overdue Balance" }
                                    td { "{account.overdue_balance_}" }
                                }
                                tr {
                                    th { class: "row-header", "Billing Name" }
                                    td { "{account.billing_name_}" }
                                }
                                tr {
                                    th { class: "row-header", "Billing Sub Name" }
                                    td { {account.billing_sub_name_.as_deref().unwrap_or("")} }
                                }
                                tr {
                                    th { class: "row-header", "Billing EMail" }
                                    td { {account.billing_email_.as_deref().unwrap_or("")} }
                                }
                            }
                        }
                    }
                }
            },
            "account" => {
                let account_user = &self.account_manager.data_set.data.viewer_;
                // let x = account_user.full_name_;
                let api_key = if let Some(api_key) = &account_user.live_secret_key_ {api_key} else {""};
                rsx! {
                    table { class: "display",
                        tr {
                            th { class: "row-header", "ID" }
                            td { "{account_user.id_}" }
                        }
                        tr {
                            th { class: "row-header", "Full Name" }
                            td { "{account_user.full_name_}" }
                        }
                        tr {
                            th { class: "row-header", "API Key" }
                            td { "{api_key}" }
                        }
                    
                    }
                }
            },
            "bills" => {
                rsx! {
                    BillsPage {
                        account_id: self.account_id.clone(),
                        bill_manager: self.bill_manager.clone(),
                        billing_timezone: self.billing_timezone,
                        path,
                    }
                }
            },
            "properties" => {
                rsx! {
                    PropertiesPage {
                        account_id: self.account_id.clone(),
                        meter_manager: self.meter_manager.clone(),
                        path,
                    }
                }
            },
            "meters" => {
                rsx! {
                    MeterPage {
                        account_id: self.account_id.clone(),
                        meter_manager: self.meter_manager.clone(),
                        billing_timezone: self.billing_timezone,
                        path,
                    }
                }
            },
            _ => {
                rsx! {
                    div { "Unknown page_id, {page_id}" }
                }
            },
        }
    }
}

/*
 * Page components.
 *
 * Each page which needs hooks is a real component so that its hooks are owned by that component
 * rather than by the enclosing Module component, which renders different pages at different times.
 */

#[derive(Props, Clone)]
struct BillsPageProps {
    account_id: String,
    bill_manager: Arc<BillManager>,
    billing_timezone: &'static Tz,
    path: Vec<String>,
}

impl PartialEq for BillsPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.bill_manager, &other.bill_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
            && self.path == other.path
    }
}

#[component]
fn BillsPage(props: BillsPageProps) -> Element {
    // Fetch the list of all bills, this runs once when the component is mounted.
    let bills_resource = use_resource({
        let bill_manager = props.bill_manager.clone();
        let account_id = props.account_id.clone();
        move || {
            let bill_manager = bill_manager.clone();
            let account_id = account_id.clone();
            async move { bill_manager.fetch_bills(account_id).await }
        }
    });

    match &*bills_resource.read() {
        None => rsx! {
            div { "Loading Bills for account {props.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load bills: {error:?}"))?,
        Some(Ok(bills)) => {
            // Are we looking at one bill?
            if let Some(bill_id) = props.path.first() {
                // The key causes BillDetail to be re-mounted when the bill changes, which drops
                // (and so cancels) any fetch in progress for the previous bill.
                rsx! {
                    BillDetail {
                        key: "{bill_id}",
                        account_id: props.account_id.clone(),
                        bill_id: bill_id.clone(),
                        bills: bills.clone(),
                        bill_manager: props.bill_manager.clone(),
                        billing_timezone: props.billing_timezone,
                    }
                }
            }
            else {
                rsx! {
                    table {
                        {AbstractBill::gui_summary_header()?}
                        for (_id , (attributes , bill)) in &bills.data_set.data {
                            {bill.gui_summary_line(attributes)?}
                        }
                    }
                }
            }
        },
    }
}

#[derive(Props, Clone)]
struct BillDetailProps {
    account_id: String,
    bill_id: String,
    bills: Arc<BillDataSet>,
    bill_manager: Arc<BillManager>,
    billing_timezone: &'static Tz,
}

impl PartialEq for BillDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.bill_id == other.bill_id
            && Arc::ptr_eq(&self.bills, &other.bills)
            && Arc::ptr_eq(&self.bill_manager, &other.bill_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn BillDetail(props: BillDetailProps) -> Element {
    // Fetch all transactions for this bill.
    let transactions_resource = use_resource({
        let props = props.clone();
        move || {
            let props = props.clone();
            async move {
                props.bill_manager.fetch_bill_transaction_breakdown(props.account_id, props.bill_id, 
                    props.billing_timezone).await
            }
        }
    });

    let Some(bill) = find_bill(&props.bill_id, &props.bills) else {
        return rsx! {
            {format!("No such bill {}", props.bill_id)}
        };
    };

    match &*transactions_resource.read() {
        None => rsx! {
            {format!("Loading transactions for bill {}...", props.bill_id)}
        },
        Some(Err(error)) => Err(anyhow!("Failed to load transactions for bill {}: {error:?}", props.bill_id))?,
        Some(Ok(bill_transactions)) => bill.gui_display(bill_transactions),
    }
}

#[derive(Props, Clone)]
struct PropertiesPageProps {
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
fn PropertiesPage(props: PropertiesPageProps) -> Element {
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



#[derive(Props, Clone)]
struct MeterPageProps {
    account_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    path: Vec<String>,
}

impl PartialEq for MeterPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
            && self.path == other.path
    }
}

#[component]
fn MeterPage(props: MeterPageProps) -> Element {
    // Fetch the list of all meters (properties), this runs once when the component is mounted.
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
            div { "Loading properties for account {props.account_id}..." }
        },
        Some(Err(error)) => Err(anyhow!("Failed to load properties: {error:?}"))?,
        Some(Ok(property_list)) => {

            // Are we looking at one meter?
            if let Some(meter_id) = props.path.first() {
            //     if let Some(property) = property_list.property_map.get(meter_id) {
            //         let p = property.clone();
                    // The key causes BillDetail to be re-mounted when the bill changes, which drops
                    // (and so cancels) any fetch in progress for the previous bill.
                    rsx! {
                        MeterConsumption {
                            key: "{meter_id}",
                            account_id: props.account_id.clone(),
                            meter_id: meter_id.clone(),
                            meter_manager: props.meter_manager.clone(),
                            billing_timezone: props.billing_timezone,
                        }
                    }
            //     }
            //     else {
            //         rsx! {
            //             div { "No such property {property_id}" }
            //         }
            //     }
            }
            else {

                let mut lines = Vec::new();

                for property in property_list.property_map.values() {
                    for meter_point in &property.electricity_meter_points_ {
                        for meter in &meter_point.meters_ {
                            let direction = if meter.import_meter_.is_some() { "Export" } else {"Import"};
                            let hh_readings = if meter.has_and_allows_hh_readings_ { "True" } else { "False" };
                            let id = meter.node_id_.clone();
                            lines.push(rsx!{
                                tr {
                                    td {
                                        div {
                                            class: "link",
                                            onclick: move |_| {

                                                let mut path_signal = use_context::<Signal<Vec<String>>>();
                                                let new_path = vec![String::from("meters"), id.clone()];
                                                path_signal.set(new_path);
                                            },
                                            "{meter.node_id_}"
                                        }
                                    }
                                    td { "{meter.serial_number_}" }
                                    td { {hh_readings} }
                                    td { "Electricity" }
                                    td { {direction} }
                                    td { {meter.consumption_units_.clone()} }
                                }
                            })
                        }
                    }


                    for meter_point in &property.gas_meter_points_ {
                        for meter in &meter_point.meters_ {
                            let direction = "Import";
                            let hh_readings = if meter.has_and_allows_hh_readings_ { "True" } else { "False" };
                            let id = meter.node_id_.clone();
                            lines.push(rsx!{
                                tr {
                                    td {
                                        div {
                                            class: "link",
                                            onclick: move |_| {

                                                let mut path_signal = use_context::<Signal<Vec<String>>>();
                                                let new_path = vec![String::from("meters"), id.clone()];
                                                path_signal.set(new_path);
                                            },
                                            "{meter.node_id_}"
                                        }
                                    }
                                    td { "{meter.serial_number_}" }
                                    td { {hh_readings} }
                                    td { "Gas" }
                                    td { {direction} }
                                    td { {meter.consumption_units_.clone()} }
                                }
                            })
                        }
                    }
                }
                rsx! {
                    table {
                        tr { class: "header",
                            th { "Node ID" }
                            th { "Serial Number" }
                            th { "HH Readings" }
                            th { "Type" }
                            th { "Direction" }
                            th { "Units" }
                        }

                        for line in lines {
                            {line}
                        }
                    }
                }
            }
        },
    }
}

#[derive(Props, Clone)]
struct MeterConsumptionProps {
    account_id: String,
    meter_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
}

impl PartialEq for MeterConsumptionProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.meter_id == other.meter_id
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

/// The number of half hour slots in a day.
const SLOTS_PER_DAY: u8 = 48;

/// Format a half hour slot as HH:MM
fn slot_label(slot: u8) -> String {
    format!("{:02}:{:02}", slot / 2, (slot % 2) * 30)
}

/// A wall clock date and half hour time slot in the billing timezone.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LocalSlot {
    date: time::Date,
    /// Half hours since midnight, 0..SLOTS_PER_DAY
    slot: u8,
}

impl LocalSlot {
    fn time(&self) -> time::Time {
        time::Time::from_hms(self.slot / 2, (self.slot % 2) * 30, 0).unwrap_or(time::Time::MIDNIGHT)
    }

    /// None if this wall clock time does not exist in the timezone, i.e. when the clocks go forward.
    /// When the clocks go back the earlier of the two possible times is used.
    fn to_date_time(&self, tz: &Tz) -> Option<DateTime> {
        let offset_date_time = time::PrimitiveDateTime::new(self.date, self.time()).assume_timezone(tz).take_first()?;
        DateTime::from_unix_timestamp(offset_date_time.unix_timestamp()).ok().map(|date_time| date_time.to_timezone(tz))
    }
}

impl std::fmt::Display for LocalSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.date, slot_label(self.slot))
    }
}

/// The selected period, an end of None means up to the present.
#[derive(Clone, Copy, PartialEq)]
struct RangeSelection {
    start: LocalSlot,
    end: Option<LocalSlot>,
}

impl RangeSelection {
    fn to_date_times(&self, tz: &Tz) -> Result<(DateTime, Option<DateTime>), String> {
        let not_exist = |slot: &LocalSlot| format!("{slot} does not exist because the clocks go forward");

        let start = self.start.to_date_time(tz).ok_or_else(|| not_exist(&self.start))?;
        let end = match &self.end {
            Some(end_slot) => {
                let end = end_slot.to_date_time(tz).ok_or_else(|| not_exist(end_slot))?;
                if end <= start {
                    return Err(String::from("The to time must be after the from time"));
                }
                Some(end)
            },
            None => None,
        };
        Ok((start, end))
    }
}

impl std::fmt::Display for RangeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.end {
            Some(end) => write!(f, "{} to {}", self.start, end),
            None => write!(f, "{} to now", self.start),
        }
    }
}

/// The values in the date range dialog, which may be incomplete while being edited.
#[derive(Clone)]
struct RangeDraft {
    start_date: Option<time::Date>,
    start_slot: u8,
    end_date: Option<time::Date>,
    end_slot: u8,
    to_now: bool,
}

impl RangeDraft {
    fn new(selection: &RangeSelection) -> Self {
        // When there is no end, offer the start date as a starting point for the user to edit.
        let end = selection.end.unwrap_or(selection.start);
        Self {
            start_date: Some(selection.start.date),
            start_slot: selection.start.slot,
            end_date: Some(end.date),
            end_slot: end.slot,
            to_now: selection.end.is_none(),
        }
    }

    fn to_selection(&self, tz: &Tz) -> Result<RangeSelection, String> {
        let start_date = self.start_date.ok_or_else(|| String::from("Select a from date"))?;
        let end = if self.to_now {
            None
        }
        else {
            let end_date = self.end_date.ok_or_else(|| String::from("Select a to date, or tick Up to now"))?;
            Some(LocalSlot { date: end_date, slot: self.end_slot })
        };
        let selection = RangeSelection { start: LocalSlot { date: start_date, slot: self.start_slot }, end };

        // Check that the times exist and are in the right order.
        selection.to_date_times(tz)?;
        Ok(selection)
    }
}

/// An empty or unparsable value clears the date.
fn parse_date_input(value: String) -> Option<time::Date> {
    value.parse::<Date>().ok().map(|date| *date)
}

/// Select element offering each half hour of the day.
fn slot_select(selected: u8, disabled: bool, onchange: impl FnMut(FormEvent) + 'static) -> Element {
    rsx! {
        select { disabled, onchange,
            for slot in 0..SLOTS_PER_DAY {
                option { value: "{slot}", selected: slot == selected, {slot_label(slot)} }
            }
        }
    }
}

/*
 * Date range selection for a meter, the selected range is passed as props to MeterDetail
 * which does the fetch and display.
 */
#[component]
fn MeterConsumption(props: MeterConsumptionProps) -> Element {
    let tz = props.billing_timezone;

    // The range currently being displayed.
    // Default to the start of the current month with no end.
    let mut applied_range = use_signal(|| {
        let today = time::OffsetDateTime::now_utc().to_timezone(tz).date();
        RangeSelection {
            start: LocalSlot { date: today.replace_day(1).unwrap_or(today), slot: 0 },
            end: None,
        }
    });

    // The contents of the date range dialog, Some while the dialog is open.
    let mut draft = use_signal::<Option<RangeDraft>>(|| None);

    let selection = *applied_range.read();
    let (start, end) = selection.to_date_times(tz).map_err(|error| anyhow!(error))?;
    let end_key = end.as_ref().map(|d| d.to_string()).unwrap_or_default();

    // The key causes MeterDetail to be re-mounted when the applied range changes, which drops
    // (and so cancels) any fetch in progress for the previous range and starts a new one.
    let detail = rsx! {
        MeterDetail {
            key: "{start}#{end_key}",
            account_id: props.account_id.clone(),
            meter_id: props.meter_id.clone(),
            meter_manager: props.meter_manager.clone(),
            billing_timezone: tz,
            start,
            end,
        }
    };

    let dialog = if let Some(current) = draft.read().clone() {
        let validated = current.to_selection(tz);
        let draft_error = validated.as_ref().err().cloned();
        let can_apply = validated.is_ok();
        let start_value = current.start_date.map(|d| d.to_string()).unwrap_or_default();
        let end_value = current.end_date.map(|d| d.to_string()).unwrap_or_default();

        // Apply a change to the draft.
        let mut edit = move |change: &dyn Fn(&mut RangeDraft)| {
            if let Some(d) = draft.write().as_mut() {
                change(d);
            }
        };

        rsx! {
            div { class: "modal-backdrop",
                form {
                    class: "modal",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        if let Ok(selection) = &validated {
                            applied_range.set(*selection);
                            draft.set(None);
                        }
                    },
                    h3 { "Select period" }
                    div { class: "modal-row",
                        label { "From" }
                        input {
                            r#type: "date",
                            value: "{start_value}",
                            onchange: move |event| edit(&|d| d.start_date = parse_date_input(event.value())),
                        }
                        {
                            slot_select(
                                current.start_slot,
                                false,
                                move |event| {
                                    if let Ok(slot) = event.value().parse::<u8>() {
                                        edit(&|d| d.start_slot = slot)
                                    }
                                },
                            )
                        }
                    }
                    div { class: "modal-row",
                        label { "To" }
                        input {
                            r#type: "date",
                            value: "{end_value}",
                            disabled: current.to_now,
                            onchange: move |event| edit(&|d| d.end_date = parse_date_input(event.value())),
                        }
                        {
                            slot_select(
                                current.end_slot,
                                current.to_now,
                                move |event| {
                                    if let Ok(slot) = event.value().parse::<u8>() {
                                        edit(&|d| d.end_slot = slot)
                                    }
                                },
                            )
                        }
                    }
                    div { class: "modal-row",
                        label {}
                        label {
                            input {
                                r#type: "checkbox",
                                checked: current.to_now,
                                onchange: move |event| edit(&|d| d.to_now = event.checked()),
                            }
                            " Up to now"
                        }
                    }
                    if let Some(error) = draft_error {
                        div { class: "modal-error", "{error}" }
                    }
                    div { class: "modal-buttons",
                        button {
                            r#type: "button",
                            onclick: move |_| draft.set(None),
                            "Cancel"
                        }
                        button { r#type: "submit", disabled: !can_apply, "Apply" }
                    }
                }
            }
        }
    }
    else {
        rsx! {}
    };

    rsx! {
        div { class: "date-range",
            button {
                r#type: "button",
                title: "Change the period",
                onclick: move |_| draft.set(Some(RangeDraft::new(&applied_range.read()))),
                Icon {
                    class: "inline-icon",
                    icon: LdCalendar,
                    width: 14,
                    height: 14,
                }
                " {selection}"
            }
        }
        {dialog}
        {detail}
    }
}

#[derive(Props, Clone)]
struct MeterDetailProps {
    // property: Arc<graphql::meter::account_properties_meters::MeterType>,
    account_id: String,
    meter_id: String,
    meter_manager: Arc<MeterManager>,
    billing_timezone: &'static Tz,
    start: DateTime,
    /// None means up to the present
    end: Option<DateTime>,
}

impl PartialEq for MeterDetailProps {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.meter_id == other.meter_id
            && self.start == other.start
            && self.end == other.end
            // && Arc::ptr_eq(&self.property, &other.property)
            && Arc::ptr_eq(&self.meter_manager, &other.meter_manager)
            && std::ptr::eq(self.billing_timezone, other.billing_timezone)
    }
}

#[component]
fn MeterDetail(props: MeterDetailProps) -> Element {
    // Fetch the data
    let transactions_resource = use_resource({
        let props = props.clone();
        move || {
            let props = props.clone();
            async move {
                props.meter_manager.get_consumption(&props.account_id, &props.meter_id,
                    &props.start, &props.end,
                    props.billing_timezone).await
                // props.bill_manager.fetch_bill_transaction_breakdown(props.account_id, props.bill_id, 
                //     props.billing_timezone).await
            }
        }
    });

    match &*transactions_resource.read() {
        None => rsx! {
            {format!("Loading consumption for meter {}...", props.meter_id)}
        },
        Some(Err(error)) => Err(anyhow!("Failed to load consumption for meter {}: {error:?}", props.meter_id))?,
        Some(Ok(consumption)) => MeterManager::display_consumption(consumption, &props.start, &props.end),
    }
}

pub struct OctopusModuleFactory {
    context: Arc<MarcoSparkoContext>,
    octopus_args: OctopusArgs,
    cache_config: Arc<CacheConfig>,
    token_manager: Arc<OctopusTokenManager>,
    request_manager: Arc<sparko_graphql::RequestManager>,
    profile: Profile,
    verbose: bool,
}

impl OctopusModuleFactory {
    pub async fn do_build(&self) -> anyhow::Result<OctopusModule> {

        let authenticated_request_manager = Arc::new(sparko_graphql::AuthenticatedRequestManager::new(self.request_manager.clone(), self.token_manager.clone())?);
       
        let client = OctopusModule::new(self.octopus_args.clone(), self.cache_config.clone(), self.profile.clone(), 
            authenticated_request_manager, self.verbose
        ).await?;

        if self.profile.init {
            crate::profile::update_profile(&self.context.profile.active_profile.name, MODULE_ID, &self.profile)?;
        }
        
        Ok(client)
    }

    async fn login(errors: &mut Vec<String>, token_manager: &Arc<OctopusTokenManager>) -> anyhow::Result<()> {

            match token_manager.get_authenticator(true).await {
                Ok(_token) => {
                    println!("Logged in OK");
                    Ok(())
                },
                Err(error) => {
                    if let sparko_graphql::Error::GraphQLError(graphql_errors) = &error {
                        let graphql_errors = &**graphql_errors;
                        for graphql_error in graphql_errors {
                            if let Some(error_code) = graphql_error.extensions.get("errorCode") {
                                if error_code == "KT-CT-1138" {
                                    errors.push(format!("Username or password is incorrect."));
                                    return Err(anyhow!(error));
                                }
                                if error_code == "KT-CT-1139" {
                                    errors.push(format!("API KEY is incorrect."));
                                    return Err(anyhow!(error));
                                }
                            }
                        }
                 
                    }
                    errors.push(format!("Login failed {}", error));
                    return Err(anyhow!(error));
                },
            }
    }

    async fn handle_login(
        token_manager: Arc<OctopusTokenManager>,
        profile: Profile,
        context: Arc<MarcoSparkoContext>,
        values: LoginForm, 
        error_signal: &mut Signal<Vec<String>>) {
        let mut errors = Vec::new();

        println!("Octopus login submitted for: {:?}", values);
        
        let login_method = values.login_method.trim().to_string();
        
        if login_method == "email" {
            let email = values.email.as_ref().unwrap_or(&String::new()).trim().to_string();
            let password = values.password.as_ref().unwrap_or(&String::new()).trim().to_string();
            
            if email.is_empty() {
                errors.push("Email is required".to_string());
            }
            if password.is_empty() {
                errors.push("Password is required".to_string());
            }

            if errors.is_empty() {
                println!("Performing login for email: {}", email);
                token_manager.set_authenticator(
                    OctopusAuthenticator::from_email_password(email.clone(), password.clone())
                ).await;

                if let Ok(_) =  Self::login(&mut errors, &token_manager).await {
                    let init_signal = try_consume_context::<Signal<InitRequested>>();
                    if let Some(mut init_sig) = init_signal {
                        init_sig.set(InitRequested(true));
                    }
                }

            }
        } else if login_method == "api_key" {
            let api_key = values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string();
            
            if api_key.is_empty() {
                errors.push("API Key is required".to_string());
            }

            if errors.is_empty() {
                println!("Performing login with API key");
                token_manager.set_authenticator(
                    OctopusAuthenticator::from_api_key(api_key.clone())
                ).await;
                if let Ok(_) = Self::login(&mut errors, &token_manager).await {
                    println!("Login successful!");
                    // Store the api_key into the profile
                    let new_profile = Profile {
                        api_key: Some(values.api_key.as_ref().unwrap_or(&String::new()).trim().to_string()),
                        ..profile.clone()
                    };

                    crate::profile::update_profile(&context.profile.active_profile.name, MODULE_ID, &new_profile).unwrap_or_else(|e| println!("profile update failed: {}", e));

                    // Reset the app initialization to reload context with new profile
                    let init_signal = try_consume_context::<Signal<InitRequested>>();
                    if let Some(mut init_sig) = init_signal {
                        init_sig.set(InitRequested(true));
                    }
                }
            }
        } else {
            errors.push("Invalid login method".to_string());
        }

        error_signal.set(errors);
    }
}

#[async_trait]
impl ModuleFactory for OctopusModuleFactory {

    async fn is_ready(&self) -> anyhow::Result<bool> {
        if let Ok(_token) = self.token_manager.get_authenticator(false).await {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn init_page(&self) -> Element {
        let mut email = use_signal(|| String::new());
        let mut password = use_signal(|| String::new());
        let mut api_key = use_signal(|| String::new());
        let mut login_method = use_signal(|| "email".to_string());
        let mut errors: Signal<Vec<String>>   = use_signal(|| Vec::new());

        let context: Arc<MarcoSparkoContext> = self.context.clone();
        let token_manager: Arc<OctopusTokenManager> = self.token_manager.clone();
        let profile: Profile = self.profile.clone();
        rsx! {
            for error in errors.read().iter() {
                div { class: "error", "{error}" }
            }
            div {
                h1 { "Octopus Login" }
                form {
                    onsubmit: move |evt: FormEvent| {
                        let context = context.clone();
                        let token_manager = token_manager.clone();
                        let profile = profile.clone();
                        async move {
                            // Prevent the default browser navigation behavior
                            evt.prevent_default();

                            // Extract the form values into the LoginForm struct
                            let values: LoginForm = evt
                                .parsed_values()
                                .expect("Failed to parse form values");

                            Self::handle_login(
                                    token_manager,
                                    profile,
                                    context.clone(),
                                    values,
                                    &mut errors,
                                )
                                .await;
                        }
                    },
                    table {
                        tr {
                            td { colspan: "2",
                                label {
                                    input {
                                        r#type: "radio",
                                        name: "login_method",
                                        value: "email",
                                        checked: login_method() == "email",
                                        onchange: move |_| login_method.set("email".to_string()),
                                    }
                                    " Login with Email and Password"
                                }
                            }
                        }
                        tr {
                            td { colspan: "2",
                                label {
                                    input {
                                        r#type: "radio",
                                        name: "login_method",
                                        value: "api_key",
                                        checked: login_method() == "api_key",
                                        onchange: move |_| login_method.set("api_key".to_string()),
                                    }
                                    " Use API Key"
                                }
                            }
                        }
                        if login_method() == "email" {
                            tr {
                                td {
                                    label { "Email:" }
                                }
                                td {
                                    input {
                                        r#type: "email",
                                        id: "email",
                                        name: "email",
                                        value: "{email}",
                                        oninput: move |e| email.set(e.value().clone()),
                                    }
                                }
                            }
                            tr {
                                td {
                                    label { "Password:" }
                                }
                                td {
                                    input {
                                        r#type: "password",
                                        id: "password",
                                        name: "password",
                                        value: "{password}",
                                        oninput: move |e| password.set(e.value().clone()),
                                    }
                                }
                            }
                        }
                        if login_method() == "api_key" {
                            tr {
                                td {
                                    label { "API Key:" }
                                }
                                td {
                                    input {
                                        r#type: "text",
                                        id: "api_key",
                                        name: "api_key",
                                        value: "{api_key}",
                                        oninput: move |e| api_key.set(e.value().clone()),
                                    }
                                }
                            }
                        }
                        tr {
                            td {
                                button { r#type: "submit", "Log In" }
                            }
                        }
                    }
                }
            }
        }
    }
    
    async fn build(&self) -> anyhow::Result<Box<dyn crate::Module + Send>> {
        Ok(Box::new(self.do_build().await?))
    }
}

pub struct OctopusModuleFactoryBuilder {
    context: Arc<MarcoSparkoContext>, 
    octopus_args: OctopusArgs,
    profile: Profile,
    authenticator: Option<OctopusAuthenticator>,
    url: Option<String>,
    verbose: bool,
}

impl OctopusModuleFactoryBuilder {
    fn new(
            context: Arc<MarcoSparkoContext>,
            json_profile: Option<serde_json::Value>
        ) -> anyhow::Result<OctopusModuleFactoryBuilder> {

        let module_args = if let Some(args) = context.args.module_args(MODULE_ID) {
            args
        }
        else {
            Vec::new()
        };

        let octopus_args = OctopusArgs::parse_from(module_args);
        let profile = if let Some(json) = json_profile {
            serde_json::from_value(json)?
        }
        else {
            Profile::new()
        };

        let option_api_key = if let Some(api_key) = &octopus_args.octopus_api_key {
            Some(api_key.to_string())
        }
        else {
            if let Some(api_key) = &profile.api_key {
                Some(api_key.to_string())
            }
            else {
                None
            }
        };

        let authenticator = if let Some(api_key) = option_api_key {
            Some(OctopusAuthenticator::from_api_key(api_key))
        }
        else {
            None
        };

        let verbose = context.args.marco_sparko_args.verbose;

        if verbose {
            println!("Octopus Module Args: {:?}", octopus_args);
            println!("Octopus Module Profile: {:?}", profile);
        }  

        Ok(OctopusModuleFactoryBuilder {
            context,
            octopus_args,
            profile,
            authenticator,
            url: None,
            verbose,
        })
    }

    // pub fn with_url(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.url = Some(url);
    //     Ok(self)
    // }

    // pub fn with_url_if_not_set(mut self, url: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     if let None = self.url {
    //         self.url = Some(url);
    //     }
    //     Ok(self)
    // }

    // pub fn with_api_key(mut self, api_key: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.authenticator = Some(OctopusAuthenticator::from_api_key(api_key));
    //     Ok(self)
    // }

    // pub fn with_password(mut self, email: String, password: String) -> anyhow::Result<OctopusModuleFactoryBuilder> {
    //     self.authenticator = Some(OctopusAuthenticator::from_email_password(email, password));
    //     Ok(self)
    // }

    pub fn build(self) -> anyhow::Result<OctopusModuleFactory> {
        let url = if let Some(url) = self.url {
            url
        }
        else {
            "https://api.octopus.energy/v1/graphql/".to_string()
        };

        let verbose = self.context.args.marco_sparko_args.verbose;
        let request_manager = Arc::new(sparko_graphql::RequestManager::new(url, verbose, create_info::USER_AGENT)?);
        let cache_config = self.context.create_cache_config(crate::octopus::MODULE_ID, verbose)?;

        Ok(OctopusModuleFactory {
            context: self.context.clone(),
            octopus_args: self.octopus_args,
            cache_config,
            token_manager: Arc::new(OctopusTokenManager::new(
                self.context,
                 request_manager.clone(),
                 self.authenticator)),
            request_manager,
            profile: self.profile,
            verbose: self.verbose,
        })
    }
}
