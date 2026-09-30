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
d dc(v N1){return dot(N1,W0(.30,.59,.11));}v E9(v ec,v F9){d G9=dc(F9);v H9=ec-dc(ec);C fc=I2(G9,1.0-G9)/max(I2(I9),I2(-v3(H9),W5(H9)));d Ze=min(M0(1.0),min(fc.x,fc.y));return H9*Ze+G9;}v gc(v e8,v hc,v F9){float af=W5(hc)-v3(hc);e8-=v3(e8);float bf=W5(e8);float J2=af/max(I9,bf);return E9(e8*J2,F9);}
#endif
v cf(v r0,i G1,R J9){v x0=P6(G1);v g1;switch(J9){case df:g1=r0.xyz*x0.xyz;break;case ef:g1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case ff:{v Q6=r0*x0;g1=2.0*mix(Q6,r0+x0-Q6-0.5,greaterThan(x0,W0(0.5)));break;}case gf:g1=min(r0.xyz,x0.xyz);break;case hf:g1=max(r0.xyz,x0.xyz);break;case jf:{G1.xyz=clamp(G1.xyz,W0(.0),G1.www);v ic=clamp(1.-r0,W0(.0),W0(1.))*G1.w;g1=mix(min(W0(1.),G1.xyz/ic),sign(G1.xyz),equal(ic,W0(.0)));break;}case lf:{r0=clamp(r0,W0(.0),W0(1.));G1.xyz=clamp(G1.xyz,W0(.0),G1.www);if(G1.w==.0) G1.w=1.;v jc=G1.w-G1.xyz;g1=1.-mix(min(W0(1.),jc/(r0*G1.w)),sign(jc),equal(r0,W0(.0)));break;}case mf:{v Q6=r0*x0;g1=2.0*mix(Q6,r0+x0-Q6-0.5,greaterThan(r0,W0(0.5)));break;}case nf:{for(int J0=0;J0<3;++J0){if(r0[J0]<=0.5) g1[J0]=(1.0-x0[J0]);else if(x0[J0]<=0.25) g1[J0]=((16.0*x0[J0]-12.0)*x0[J0]+3.0);else g1[J0]=(inversesqrt(x0[J0])-1.0);}g1=x0+x0*(2.0*r0-1.0)*g1;break;}case of:g1=abs(x0.xyz-r0.xyz);break;case pf:g1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef ENABLE_HSL_BLEND_MODES
case qf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=gc(r0.xyz,x0.xyz,x0.xyz);}break;case rf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=gc(x0.xyz,r0.xyz,x0.xyz);}break;case sf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=E9(r0.xyz,x0.xyz);}break;case tf:if(ENABLE_HSL_BLEND_MODES){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=E9(x0.xyz,r0.xyz);}break;
#endif
}return g1;}f v h5(v r0,i G1,R J9){v g1=cf(r0,G1,J9);return mix(r0,g1,W0(G1.w));}
#endif
#endif
