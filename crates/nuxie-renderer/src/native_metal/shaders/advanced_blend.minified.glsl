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
d Hc(v S1){return dot(S1,a1(.30,.59,.11));}v da(v Ic,v ea){d fa=Hc(ea);v ga=Ic-Hc(Ic);D Jc=R2(fa,1.0-fa)/max(R2(ha),R2(-A3(ga),e6(ga)));d Ff=min(I0(1.0),min(Jc.x,Jc.y));return ga*Ff+fa;}v Kc(v x8,v Lc,v ea){float Gf=e6(Lc)-A3(Lc);x8-=A3(x8);float Hf=e6(x8);float S2=Gf/max(ha,Hf);return da(x8*S2,ea);}
#endif
v If(v r0,i J1,P g4){v x0=f6(J1);v k1;switch(g4){case Jf:k1=r0.xyz*x0.xyz;break;case Kf:k1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case Lf:{v Z6=r0*x0;k1=2.0*mix(Z6,r0+x0-Z6-0.5,greaterThan(x0,a1(0.5)));break;}case Mf:k1=min(r0.xyz,x0.xyz);break;case Nf:k1=max(r0.xyz,x0.xyz);break;case Of:{J1.xyz=clamp(J1.xyz,a1(.0),J1.www);v Mc=clamp(1.-r0,a1(.0),a1(1.))*J1.w;k1=mix(min(a1(1.),J1.xyz/Mc),sign(J1.xyz),equal(Mc,a1(.0)));break;}case Qf:{r0=clamp(r0,a1(.0),a1(1.));J1.xyz=clamp(J1.xyz,a1(.0),J1.www);if(J1.w==.0) J1.w=1.;v Nc=J1.w-J1.xyz;k1=1.-mix(min(a1(1.),Nc/(r0*J1.w)),sign(Nc),equal(r0,a1(.0)));break;}case Rf:{v Z6=r0*x0;k1=2.0*mix(Z6,r0+x0-Z6-0.5,greaterThan(r0,a1(0.5)));break;}case Sf:{for(int M0=0;M0<3;++M0){if(r0[M0]<=0.5) k1[M0]=(1.0-x0[M0]);else if(x0[M0]<=0.25) k1[M0]=((16.0*x0[M0]-12.0)*x0[M0]+3.0);else k1[M0]=(inversesqrt(x0[M0])-1.0);}k1=x0+x0*(2.0*r0-1.0)*k1;break;}case Tf:k1=abs(x0.xyz-r0.xyz);break;case Uf:k1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef ENABLE_HSL_BLEND_MODES
case Vf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=Kc(r0.xyz,x0.xyz,x0.xyz);}break;case Wf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=Kc(x0.xyz,r0.xyz,x0.xyz);}break;case Xf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=da(r0.xyz,x0.xyz);}break;case Yf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,a1(.0),a1(1.));k1=da(x0.xyz,r0.xyz);}break;
#endif
}return k1;}f v L4(v r0,i J1,P g4){v k1=If(r0,J1,g4);return mix(r0,k1,a1(J1.w));}
#endif
#endif
