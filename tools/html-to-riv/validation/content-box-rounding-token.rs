// Standalone diagnosis against the module's already-built cssparser dependency.
// No compiler/runtime source or dependency is changed.
use cssparser::{Parser,ParserInput,Token,ToCss};
fn main(){
    for source in ["999999.875px","999999.9375px","62499.9921875rem","1000000px",".03125px","100%"] {
        let mut input=ParserInput::new(source);let mut parser=Parser::new(&mut input);
        let token=parser.next().unwrap().clone();
        let scalar=match &token {Token::Dimension{value,..}|Token::Number{value,..}=>f64::from(*value),Token::Percentage{unit_value,..}=>f64::from(*unit_value),_=>panic!()};
        println!("source={source}\tparsed_binary32_as_f64={scalar}\tserialized={}",token.to_css_string());
    }
}
