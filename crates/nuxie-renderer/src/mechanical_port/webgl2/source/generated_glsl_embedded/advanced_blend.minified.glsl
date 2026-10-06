#ifdef FB
#ifdef LE
layout(
#ifdef FC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
) out;
#endif
#ifdef N
#ifdef FC
d ec(v P1){return dot(P1,W0(.30,.59,.11));}v G9(v fc,v H9){d I9=ec(H9);v J9=fc-ec(fc);C gc=H2(I9,1.0-I9)/max(H2(K9),H2(-w3(J9),Y5(J9)));d ef=min(H0(1.0),min(gc.x,gc.y));return J9*ef+I9;}v hc(v e8,v ic,v H9){float ff=Y5(ic)-w3(ic);e8-=w3(e8);float gf=Y5(e8);float I2=ff/max(K9,gf);return G9(e8*I2,H9);}
#endif
v hf(v r0,i H1,Q c4){v x0=Q6(H1);v g1;switch(c4){case jf:g1=r0.xyz*x0.xyz;break;case kf:g1=r0.xyz+x0.xyz-r0.xyz*x0.xyz;break;case lf:{v R6=r0*x0;g1=2.0*mix(R6,r0+x0-R6-0.5,greaterThan(x0,W0(0.5)));break;}case mf:g1=min(r0.xyz,x0.xyz);break;case nf:g1=max(r0.xyz,x0.xyz);break;case of:{H1.xyz=clamp(H1.xyz,W0(.0),H1.www);v jc=clamp(1.-r0,W0(.0),W0(1.))*H1.w;g1=mix(min(W0(1.),H1.xyz/jc),sign(H1.xyz),equal(jc,W0(.0)));break;}case qf:{r0=clamp(r0,W0(.0),W0(1.));H1.xyz=clamp(H1.xyz,W0(.0),H1.www);if(H1.w==.0) H1.w=1.;v kc=H1.w-H1.xyz;g1=1.-mix(min(W0(1.),kc/(r0*H1.w)),sign(kc),equal(r0,W0(.0)));break;}case rf:{v R6=r0*x0;g1=2.0*mix(R6,r0+x0-R6-0.5,greaterThan(r0,W0(0.5)));break;}case sf:{for(int L0=0;L0<3;++L0){if(r0[L0]<=0.5) g1[L0]=(1.0-x0[L0]);else if(x0[L0]<=0.25) g1[L0]=((16.0*x0[L0]-12.0)*x0[L0]+3.0);else g1[L0]=(inversesqrt(x0[L0])-1.0);}g1=x0+x0*(2.0*r0-1.0)*g1;break;}case tf:g1=abs(x0.xyz-r0.xyz);break;case uf:g1=r0.xyz+x0.xyz-2.*r0.xyz*x0.xyz;break;
#ifdef FC
case vf:if(FC){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=hc(r0.xyz,x0.xyz,x0.xyz);}break;case wf:if(FC){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=hc(x0.xyz,r0.xyz,x0.xyz);}break;case xf:if(FC){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=G9(r0.xyz,x0.xyz);}break;case yf:if(FC){r0.xyz=clamp(r0.xyz,W0(.0),W0(1.));g1=G9(x0.xyz,r0.xyz);}break;
#endif
}return g1;}f v h5(v r0,i H1,Q c4){v g1=hf(r0,H1,c4);return mix(r0,g1,W0(H1.w));}
#endif
#endif
