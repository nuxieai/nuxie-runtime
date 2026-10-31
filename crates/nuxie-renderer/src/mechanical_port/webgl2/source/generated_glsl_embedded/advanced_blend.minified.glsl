#ifdef EB
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
d cd(v Y1){return dot(Y1,d1(.30,.59,.11));}v qa(v dd,v ra){d sa=cd(ra);v ta=dd-cd(dd);F ed=U2(sa,1.0-sa)/max(U2(ua),U2(-F3(ta),l6(ta)));d bg=min(A0(1.0),min(ed.x,ed.y));return ta*bg+sa;}v fd(v I8,v gd,v ra){float cg=l6(gd)-F3(gd);I8-=F3(I8);float dg=l6(I8);float V2=cg/max(ua,dg);return qa(I8*V2,ra);}
#endif
v eg(v v0,i A1,O G3){v B0=m6(A1);v k1;switch(G3){case fg:k1=v0.xyz*B0.xyz;break;case gg:k1=v0.xyz+B0.xyz-v0.xyz*B0.xyz;break;case hg:{v i7=v0*B0;k1=2.0*mix(i7,v0+B0-i7-0.5,greaterThan(B0,d1(0.5)));break;}case ig:k1=min(v0.xyz,B0.xyz);break;case jg:k1=max(v0.xyz,B0.xyz);break;case kg:{A1.xyz=clamp(A1.xyz,d1(.0),A1.www);v hd=clamp(1.-v0,d1(.0),d1(1.))*A1.w;k1=mix(min(d1(1.),A1.xyz/hd),sign(A1.xyz),equal(hd,d1(.0)));break;}case mg:{v0=clamp(v0,d1(.0),d1(1.));A1.xyz=clamp(A1.xyz,d1(.0),A1.www);if(A1.w==.0) A1.w=1.;v id=A1.w-A1.xyz;k1=1.-mix(min(d1(1.),id/(v0*A1.w)),sign(id),equal(v0,d1(.0)));break;}case ng:{v i7=v0*B0;k1=2.0*mix(i7,v0+B0-i7-0.5,greaterThan(v0,d1(0.5)));break;}case og:{for(int P0=0;P0<3;++P0){if(v0[P0]<=0.5) k1[P0]=(1.0-B0[P0]);else if(B0[P0]<=0.25) k1[P0]=((16.0*B0[P0]-12.0)*B0[P0]+3.0);else k1[P0]=(inversesqrt(B0[P0])-1.0);}k1=B0+B0*(2.0*v0-1.0)*k1;break;}case pg:k1=abs(B0.xyz-v0.xyz);break;case qg:k1=v0.xyz+B0.xyz-2.*v0.xyz*B0.xyz;break;
#ifdef GC
case rg:if(GC){v0.xyz=clamp(v0.xyz,d1(.0),d1(1.));k1=fd(v0.xyz,B0.xyz,B0.xyz);}break;case sg:if(GC){v0.xyz=clamp(v0.xyz,d1(.0),d1(1.));k1=fd(B0.xyz,v0.xyz,B0.xyz);}break;case tg:if(GC){v0.xyz=clamp(v0.xyz,d1(.0),d1(1.));k1=qa(v0.xyz,B0.xyz);}break;case ug:if(GC){v0.xyz=clamp(v0.xyz,d1(.0),d1(1.));k1=qa(B0.xyz,v0.xyz);}break;
#endif
}return k1;}f v X4(v v0,i A1,O G3){v k1=eg(v0,A1,G3);return mix(v0,k1,d1(A1.w));}
#endif
#endif
