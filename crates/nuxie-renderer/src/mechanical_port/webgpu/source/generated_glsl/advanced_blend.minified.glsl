#ifdef FRAGMENT
#ifdef ENABLE_KHR_BLEND
layout(
#ifdef ENABLE_HSL_BLEND_MODES
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef ENABLE_ADVANCED_BLEND
#ifdef ENABLE_HSL_BLEND_MODES
d Qc(v T1){return dot(T1,Z0(.30,.59,.11));}v ma(v Rc,v na){d oa=Qc(na);v pa=Rc-Qc(Rc);D Sc=R2(oa,1.0-oa)/max(R2(qa),R2(-B3(pa),h6(pa)));d Kf=min(J0(1.0),min(Sc.x,Sc.y));return pa*Kf+oa;}v Tc(v D8,v Uc,v na){float Lf=h6(Uc)-B3(Uc);D8-=B3(D8);float Mf=h6(D8);float S2=Lf/max(qa,Mf);return ma(D8*S2,na);}
#endif
v Nf(v r0,i x1,P h4){v y0=i6(x1);v j1;switch(h4){case Of:j1=r0.xyz*y0.xyz;break;case Pf:j1=r0.xyz+y0.xyz-r0.xyz*y0.xyz;break;case Qf:{v i7=r0*y0;j1=2.0*mix(i7,r0+y0-i7-0.5,greaterThan(y0,Z0(0.5)));break;}case Rf:j1=min(r0.xyz,y0.xyz);break;case Sf:j1=max(r0.xyz,y0.xyz);break;case Tf:{x1.xyz=clamp(x1.xyz,Z0(.0),x1.www);v Vc=clamp(1.-r0,Z0(.0),Z0(1.))*x1.w;j1=mix(min(Z0(1.),x1.xyz/Vc),sign(x1.xyz),equal(Vc,Z0(.0)));break;}case Vf:{r0=clamp(r0,Z0(.0),Z0(1.));x1.xyz=clamp(x1.xyz,Z0(.0),x1.www);if(x1.w==.0) x1.w=1.;v Wc=x1.w-x1.xyz;j1=1.-mix(min(Z0(1.),Wc/(r0*x1.w)),sign(Wc),equal(r0,Z0(.0)));break;}case Wf:{v i7=r0*y0;j1=2.0*mix(i7,r0+y0-i7-0.5,greaterThan(r0,Z0(0.5)));break;}case Xf:{for(int M0=0;M0<3;++M0){if(r0[M0]<=0.5) j1[M0]=(1.0-y0[M0]);else if(y0[M0]<=0.25) j1[M0]=((16.0*y0[M0]-12.0)*y0[M0]+3.0);else j1[M0]=(inversesqrt(y0[M0])-1.0);}j1=y0+y0*(2.0*r0-1.0)*j1;break;}case Yf:j1=abs(y0.xyz-r0.xyz);break;case Zf:j1=r0.xyz+y0.xyz-2.*r0.xyz*y0.xyz;break;
#ifdef ENABLE_HSL_BLEND_MODES
case ag:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));j1=Tc(r0.xyz,y0.xyz,y0.xyz);}break;case bg:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));j1=Tc(y0.xyz,r0.xyz,y0.xyz);}break;case cg:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));j1=ma(r0.xyz,y0.xyz);}break;case dg:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,Z0(.0),Z0(1.));j1=ma(y0.xyz,r0.xyz);}break;
#endif
}return j1;}e v N4(v r0,i x1,P h4){v j1=Nf(r0,x1,h4);return mix(r0,j1,Z0(x1.w));}
#endif
#endif
