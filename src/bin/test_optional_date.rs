use sparko_graphql::types::{Date, DateTime};

fn disp(a: &Option<DateTime>) -> String{
    if let Some(d) = a {
        d.to_string()
    }
    else {
        String::from("None")
    }
}

fn test(a: &Option<DateTime>, b: &Option<DateTime>) {
    let cmp = a < b;

    println!("{} < {} = {}", disp(a), disp(b), cmp);
}

// fn test2(a: &Option<DateTime>, b: &Option<Date>) {
//     let cmp = a < b;

//     println!("{} < {} = {}", disp(a), disp(b), cmp);
// }

fn main() {
    let a = Some(DateTime::from_calendar_date(2020, time::Month::January, 1).unwrap());

    let b = Some(DateTime::from_calendar_date(2024, time::Month::January, 1).unwrap());

    test(&a, &b);
    test(&b, &b);
    test(&b, &a);
    test(&a, &None);
    test(&None, &b);
    test(&None, &a);

    test(&None, &Some(DateTime::from_unix_timestamp(0).unwrap()));
    test(&None, &Some(DateTime::from_unix_timestamp(-99999999).unwrap()));
    test(&None, &None);

    // let d1 = Some(Date::from_calendar_date(2020, time::Month::January, 1).unwrap());
    // let dt1 = Some(DateTime::from_calendar_date_time(2020, time::Month::January, 1, 14, 20, 20).unwrap());

    // test2(&dt1, &d1);
}