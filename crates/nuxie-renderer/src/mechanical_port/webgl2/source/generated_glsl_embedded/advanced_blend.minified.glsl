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
d Zc(v W1){return dot(W1,Y0(.30,.59,.11));}v na(v ad,v oa){d pa=Zc(oa);v qa=ad-Zc(ad);E bd=U2(pa,1.0-pa)/max(U2(ra),U2(-D3(qa),i6(qa)));d Uf=min(C0(1.0),min(bd.x,bd.y));return qa*Uf+pa;}v cd(v E8,v dd,v oa){float Vf=i6(dd)-D3(dd);E8-=D3(E8);float Wf=i6(E8);float V2=Vf/max(ra,Wf);return na(E8*V2,oa);}
#endif
v Xf(v v0,i z1,O E3){v z0=j6(z1);v i1;switch(E3){case Yf:i1=v0.xyz*z0.xyz;break;case Zf:i1=v0.xyz+z0.xyz-v0.xyz*z0.xyz;break;case ag:{v i7=v0*z0;i1=2.0*mix(i7,v0+z0-i7-0.5,greaterThan(z0,Y0(0.5)));break;}case bg:i1=min(v0.xyz,z0.xyz);break;case cg:i1=max(v0.xyz,z0.xyz);break;case dg:{z1.xyz=clamp(z1.xyz,Y0(.0),z1.www);v ed=clamp(1.-v0,Y0(.0),Y0(1.))*z1.w;i1=mix(min(Y0(1.),z1.xyz/ed),sign(z1.xyz),equal(ed,Y0(.0)));break;}case fg:{v0=clamp(v0,Y0(.0),Y0(1.));z1.xyz=clamp(z1.xyz,Y0(.0),z1.www);if(z1.w==.0) z1.w=1.;v fd=z1.w-z1.xyz;i1=1.-mix(min(Y0(1.),fd/(v0*z1.w)),sign(fd),equal(v0,Y0(.0)));break;}case gg:{v i7=v0*z0;i1=2.0*mix(i7,v0+z0-i7-0.5,greaterThan(v0,Y0(0.5)));break;}case hg:{for(int P0=0;P0<3;++P0){if(v0[P0]<=0.5) i1[P0]=(1.0-z0[P0]);else if(z0[P0]<=0.25) i1[P0]=((16.0*z0[P0]-12.0)*z0[P0]+3.0);else i1[P0]=(inversesqrt(z0[P0])-1.0);}i1=z0+z0*(2.0*v0-1.0)*i1;break;}case ig:i1=abs(z0.xyz-v0.xyz);break;case jg:i1=v0.xyz+z0.xyz-2.*v0.xyz*z0.xyz;break;
#ifdef GC
case kg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=cd(v0.xyz,z0.xyz,z0.xyz);}break;case lg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=cd(z0.xyz,v0.xyz,z0.xyz);}break;case mg:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=na(v0.xyz,z0.xyz);}break;case ng:if(GC){v0.xyz=clamp(v0.xyz,Y0(.0),Y0(1.));i1=na(z0.xyz,v0.xyz);}break;
#endif
}return i1;}f v R4(v v0,i z1,O E3){v i1=Xf(v0,z1,E3);return mix(v0,i1,Y0(z1.w));}
#endif
#endif
