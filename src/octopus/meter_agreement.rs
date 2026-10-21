use std::collections::HashMap;
use std::time::Duration;

use sparko_graphql::{GraphQLCaller, GraphQLQueryBuilder};
use sparko_graphql::types::DateTime;

use crate::CacheConfig;
use crate::data_set::MultiQueryDataSet;

use super::graphql::meter;
use super::meter::{MeterType, Tariff};

/// An agreement, which is the tariff for a meter over a period of time.
#[derive(Debug)]
pub struct MeterAgreement {
    pub agreement_id: String,
    pub tariff: Tariff,
}

pub struct MeterAgreementDataSet {
    pub import_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub export_electricity_map: HashMap<String, Vec<meter::meter_agreements::ElectricityAgreementType>>,
    pub gas_map: HashMap<String, Vec<meter::meter_agreements::GasAgreementType>>,
}

impl MeterAgreementDataSet {
    pub async fn new(config: CacheConfig, request_manager: &impl GraphQLCaller, account_number: &str, meter_node_ids: &Vec<String>) -> anyhow::Result<Self> {
        let hash_key = format!("{}#MeterAgreements", account_number);
        let the_beginning: DateTime = DateTime::from_calendar_date(2000, time::Month::January, 1)?;
        let mut queries = Vec::new();

        for meter_node_id in meter_node_ids {
            let query = meter::meter_agreements::Query::builder()
                .with_meter_node_id(meter_node_id.clone())
                .with_valid_after(the_beginning.clone())
                .build();

            queries.push(query);
        }
        let data = MultiQueryDataSet::new(
            &hash_key,
            Duration::from_hours(24),
            queries,
            config,
            request_manager).await?;
        

        let mut export_electricity_map = HashMap::new();
        let mut import_electricity_map = HashMap::new();
        let mut gas_map = HashMap::new();

        for response in data.data {
            match response.node_ {
                meter::meter_agreements::Node::ElectricityMeterType(electricity_meter_type) => {
                    if electricity_meter_type.import_meter_.is_some() {
                        export_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                    else {
                        import_electricity_map.insert(electricity_meter_type.node_id_, electricity_meter_type.meter_point_.agreements_);
                    }
                },
                meter::meter_agreements::Node::GasMeterType(gas_meter_type) => {
                    gas_map.insert(gas_meter_type.node_id_, gas_meter_type.meter_point_.agreements_);
                },
                _ => unreachable!()
            }
        }
        
        Ok(Self {
            export_electricity_map,
            import_electricity_map,
            gas_map,
        })
    }

    pub fn get_in_scope(self, meter_type: &MeterType, is_export: bool, start_date: &DateTime, end_date: &DateTime) -> Vec<MeterAgreement> {
        let mut in_scope_agreements = Vec::new();

        match meter_type {
            MeterType::Gas => {
                for (_meter_node_id, agreement_vec) in self.gas_map {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push(MeterAgreement { agreement_id: agreement.id_.to_string(), tariff: Tariff::Gas(agreement.tariff_) });
                                }
                            }
                            else {
                                in_scope_agreements.push(MeterAgreement { agreement_id: agreement.id_.to_string(), tariff: Tariff::Gas(agreement.tariff_) });
                            }
                        }
                    }
                }
            },
            MeterType::Electricity => {
                for (_meter_node_id, agreement_vec) in if is_export {self.export_electricity_map} else {self.import_electricity_map} {
                    for agreement in agreement_vec {
                        if &agreement.valid_from_ <= end_date {
                            if let Some(valid_to) = &agreement.valid_to_ {
                                if valid_to >= start_date {
                                    in_scope_agreements.push(MeterAgreement { agreement_id: agreement.id_.to_string(), tariff: Tariff::Electricity(agreement.tariff_) });
                                }
                            }
                            else {
                                in_scope_agreements.push(MeterAgreement { agreement_id: agreement.id_.to_string(), tariff: Tariff::Electricity(agreement.tariff_) });
                            }
                        }
                    }
                }
            },
        }

        in_scope_agreements
    }
}
