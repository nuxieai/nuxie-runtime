#ifdef FRAGMENT
#ifdef ENABLE_KHR_BLEND
layout(
#ifdef ENABLE_HSL_BLEND_MODES
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef ENABLE_ADVANCED_BLEND
#ifdef ENABLE_HSL_BLEND_MODES
c Bb(A H1){return dot(H1,Q0(.30,.59,.11));}A l9(A Cb,A m9){c n9=Bb(m9);A o9=Cb-Bb(Cb);E Db=B2(n9,1.0-n9)/max(B2(p9),B2(-h3(o9),N5(o9)));c we=min(G0(1.0),min(Db.x,Db.y));return o9*we+n9;}A Eb(A R7,A Fb,A m9){float xe=N5(Fb)-h3(Fb);R7-=h3(R7);float ye=N5(R7);float C2=xe/max(p9,ye);return l9(R7*C2,m9);}
#endif
A ze(A n0,i y1,N q9){A q0=H6(y1);A X0;switch(q9){case Ae:X0=n0.xyz*q0.xyz;break;case Be:X0=n0.xyz+q0.xyz-n0.xyz*q0.xyz;break;case Ce:{A I6=n0*q0;X0=2.0*mix(I6,n0+q0-I6-0.5,greaterThan(q0,Q0(0.5)));break;}case De:X0=min(n0.xyz,q0.xyz);break;case Ee:X0=max(n0.xyz,q0.xyz);break;case Fe:{y1.xyz=clamp(y1.xyz,Q0(.0),y1.www);A Gb=clamp(1.-n0,Q0(.0),Q0(1.))*y1.w;X0=mix(min(Q0(1.),y1.xyz/Gb),sign(y1.xyz),equal(Gb,Q0(.0)));break;}case He:{n0=clamp(n0,Q0(.0),Q0(1.));y1.xyz=clamp(y1.xyz,Q0(.0),y1.www);if(y1.w==.0)y1.w=1.;A Hb=y1.w-y1.xyz;X0=1.-mix(min(Q0(1.),Hb/(n0*y1.w)),sign(Hb),equal(n0,Q0(.0)));break;}case Ie:{A I6=n0*q0;X0=2.0*mix(I6,n0+q0-I6-0.5,greaterThan(n0,Q0(0.5)));break;}case Je:{for(int F0=0;F0<3;++F0){if(n0[F0]<=0.5)X0[F0]=(1.0-q0[F0]);else if(q0[F0]<=0.25)X0[F0]=((16.0*q0[F0]-12.0)*q0[F0]+3.0);else X0[F0]=(inversesqrt(q0[F0])-1.0);}X0=q0+q0*(2.0*n0-1.0)*X0;break;}case Ke:X0=abs(q0.xyz-n0.xyz);break;case Le:X0=n0.xyz+q0.xyz-2.*n0.xyz*q0.xyz;break;
#ifdef ENABLE_HSL_BLEND_MODES
case Me:if(ENABLE_HSL_BLEND_MODES){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Eb(n0.xyz,q0.xyz,q0.xyz);}break;case Ne:if(ENABLE_HSL_BLEND_MODES){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Eb(q0.xyz,n0.xyz,q0.xyz);}break;case Oe:if(ENABLE_HSL_BLEND_MODES){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=l9(n0.xyz,q0.xyz);}break;case Pe:if(ENABLE_HSL_BLEND_MODES){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=l9(q0.xyz,n0.xyz);}break;
#endif
}return X0;}e A V4(A n0,i y1,N q9){A X0=ze(n0,y1,q9);return mix(n0,X0,Q0(y1.w));}
#endif
#endif
