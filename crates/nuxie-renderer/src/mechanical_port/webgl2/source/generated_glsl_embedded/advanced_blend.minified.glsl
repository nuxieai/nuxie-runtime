#ifdef DB
#ifdef RE
layout(
#ifdef GC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef H
#ifdef GC
d Yc(v W1){return dot(W1,Y0(.30,.59,.11));}v na(v Zc,v oa){d pa=Yc(oa);v qa=Zc-Yc(Zc);E ad=U2(pa,1.0-pa)/max(U2(ra),U2(-D3(qa),i6(qa)));d Tf=min(C0(1.0),min(ad.x,ad.y));return qa*Tf+pa;}v bd(v D8,v cd,v oa){float Uf=i6(cd)-D3(cd);D8-=D3(D8);float Vf=i6(D8);float V2=Uf/max(ra,Vf);return na(D8*V2,oa);}
#endif
v Wf(v v0,i z1,O E3){v z0=j6(z1);v i1;switch(E3){case Xf:i1=v0.xyz*z0.xyz;break;case Yf:i1=v0.xyz+z0.xyz-v0.xyz*z0.xyz;break;case Zf:{v g7=v0*z0;i1=2.0*mix(g7,v0+z0-g7-0.5,greaterThan(z0,Y0(0.5)));break;}case ag:i1=min(v0.xyz,z0.xyz);break;case bg:i1=max(v0.xyz,z0.xyz);break;case cg:{z1.xyz=clamp(z1.xyz,Y0(.0),z1.www);v dd=clamp(1.-v0,Y0(.0),Y0(1.))*z1.w;i1=mix(min(Y0(1.),z1.xyz/dd),sign(z1.xyz),equal(dd,Y0(.0)));break;}case eg:{v0=clamp(v0,Y0(.0),Y0(1.));z1.xyz=clamp(z1.xyz,Y0(.0),z1.www);if(z1.w==.0) z1.w=1.;v ed=z1.w-z1.xyz;i1=1.-mix(min(Y0(1.),ed/(v0*z1.w)),sign(ed),equal(v0,Y0(.0)));break;}case fg:{v g7=v0*z0;i1=2.0*mix(g7,v0+z0-g7-0.5,greaterThan(v0,Y0(0.5)));break;}case gg:{for(int P0=0;P0<3;++P0){if(v0[P0]<=0.5) i1[P0]=(1.0-z0[P0]);else if(z0[P0]<=0.25) i1[P0]=((16.0*z0[P0]-12.0)*z0[P0]+3.0);else i1[P0]=(inversesqrt(z0[P0])-1.0);}i1=z0+z0*(2.0*v0-1.0)*i1;break;}case hg:i1=abs(z0.xyz-v0.xyz);break;case ig:i1=v0.xyz+z0.xyz-2.*v0.xyz*z0.xyz;break;
#ifdef GC
case jg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=bd(v0.xyz,z0.xyz,z0.xyz);}break;case kg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=bd(z0.xyz,v0.xyz,z0.xyz);}break;case lg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=na(v0.xyz,z0.xyz);}break;case mg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=na(z0.xyz,v0.xyz);}break;
#endif
}return i1;}f v R4(v v0,i z1,O E3){v i1=Wf(v0,z1,E3);return mix(v0,i1,Y0(z1.w));}
#endif
#endif
