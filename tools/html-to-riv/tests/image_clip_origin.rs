use nuxie_html_to_riv::{compile,Asset,CompileInput,OverflowClipBox};
fn origin(css:&str)->Option<(OverflowClipBox,f32)> {
 let result=compile(&CompileInput {html:"<div><img src='asset:photo'></div>".into(),css:css.into(),width:390.,height:320.,
 assets:[("photo".into(),Asset::Image{bytes:include_bytes!("assets/quadrants.png").to_vec()})].into(),..Default::default()}).unwrap();
 result.runtime_requirements.layout_overflow_clip_margins.first().map(|e|(e.origin,e.pixels))
}
#[test]
fn replaced_images_default_to_content_clip_but_authored_values_override() {
 for overflow in ["hidden","clip"] {
  let css=format!("img{{width:100px;height:90px;padding:8px 12px;overflow:{overflow};border-radius:24px}}");
  assert_eq!(origin(&css),Some((OverflowClipBox::ContentBox,0.)));
  for value in ["initial","unset","padding-box","0px"] {
   assert_eq!(origin(&format!("{css} img{{overflow-clip-margin:{value}}}")),None);
  }
  assert_eq!(origin(&format!("{css} img{{overflow-clip-margin:8px}}")),Some((OverflowClipBox::PaddingBox,8.)));
  assert_eq!(origin(&format!("{css} div{{overflow-clip-margin:border-box 3px}} img{{overflow-clip-margin:inherit}}")),Some((OverflowClipBox::BorderBox,3.)));
  assert_eq!(origin(&format!("{css} img{{--clip:content-box 4px;overflow-clip-margin:var(--clip)}}")),Some((OverflowClipBox::ContentBox,4.)));
 }
 assert_eq!(origin("img{overflow:visible;border-radius:24px}"),None);
}
