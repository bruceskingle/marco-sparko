use marco_sparko::MarcoSparko;
use time::macros::datetime;
use time::macros::offset;
use time_tz::timezones;
use time_tz::OffsetDateTimeExt;
use std::str::FromStr;

#[tokio::main]
async fn main() {

    // // let time_zone = tzdb::tz_by_name("Europe/Berlin").unwrap();
    // // let current_time: tz::DateTime = tzdb::now::in_named("Europe/Berlin").unwrap();

    // let london: &time_tz::Tz = timezones::get_by_name("Europe/London").unwrap();
    // let d = sparko_graphql::types::DateTime::from_str("2022-08-21T10:00:00+01:00").unwrap();

    // let d2 = d.to_date().at_midnight().to_timezone(london);

    // println!("d={}", d);
    // println!("d2={}", d2);



    // // let odt1: time::OffsetDateTime = datetime!(2021-01-01 12:0:0 UTC);
    // //     assert_eq!(odt1.to_timezone(london), datetime!(2021-01-01 12:0:0 +0));
    // //     let odt2: time::OffsetDateTime = datetime!(2021-07-01 12:0:0 UTC);
    // //     // Adding offset to datetime call causes VERY surprising result: hours randomly changes!!
    // //     // When using UTC followed by .to_offset no surprising result.
    // //     assert_eq!(
    // //         odt2.to_timezone(london),
    // //         datetime!(2021-07-01 12:0:0 UTC).to_offset(offset!(+1))
    // //     );

    // //     let current_time = odt2.to_timezone(london);

    // // println!("current_time = {}", current_time);
    // panic!();


    match MarcoSparko::new().await {
        Ok(ms) => {
            let mut marco_sparko = ms; //Box::new(ms);

            if let Err(error) = marco_sparko.run().await {
                println!("Execution failed: {}", error);
            }
        },
        Err(error) => println!("Initialization failed: {}", error),
    }
}