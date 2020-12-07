use restq::{
    ast::{
        ddl::*,
        dml::*,
        *,
    },
    *,
};

#[test]
fn complex_select() {
    let input = "person?age=lt.42&(student=eq.true|gender=eq.'M')&group_by=sum(age),grade,gender&having=min(age)=gt.42&order_by=age.desc,height.asc&page=20&page_size=100";
    let input_chars = to_chars(input);
    let ret = select().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(input, ret.to_string());
}

#[test]
fn complex_select_with_projection() {
    let input = "person{person_id,name,updated}?age=lt.42&(student=eq.true|gender=eq.'M')&group_by=sum(age),grade,gender&having=min(age)=gt.42&order_by=age.desc,height.asc&page=20&page_size=100";
    let input_chars = to_chars(input);
    let ret = select().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(input, ret.to_string());
}

#[test]
fn complex_select_with_filter_in() {
    let input = "person{person_id,name,updated}?person_id=in.[100,101,102]";
    let input_chars = to_chars(input);
    let ret = select().parse(&input_chars).expect("must be parsed");
    println!("ret: {:#?}", ret);
    assert_eq!(input, ret.to_string());
}
