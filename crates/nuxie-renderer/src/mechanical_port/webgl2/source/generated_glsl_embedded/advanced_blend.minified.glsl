#ifdef FB
#ifdef ME
layout(
#ifdef EC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef AB
#ifdef EC
d Db(A G1){return dot(G1,R0(.30,.59,.11));}A m9(A Eb,A n9){d o9=Db(n9);A p9=Eb-Db(Eb);E Fb=D2(o9,1.0-o9)/max(D2(q9),D2(-i3(p9),M5(p9)));d He=min(I0(1.0),min(Fb.x,Fb.y));return p9*He+o9;}A Gb(A R7,A Hb,A n9){float Ie=M5(Hb)-i3(Hb);R7-=i3(R7);float Je=M5(R7);float E2=Ie/max(q9,Je);return m9(R7*E2,n9);}
#endif
A Ke(A n0,i x1,L r9){A r0=H6(x1);A X0;switch(r9){case Le:X0=n0.xyz*r0.xyz;break;case Me:X0=n0.xyz+r0.xyz-n0.xyz*r0.xyz;break;case Ne:{A I6=n0*r0;X0=2.0*mix(I6,n0+r0-I6-0.5,greaterThan(r0,R0(0.5)));break;}case Oe:X0=min(n0.xyz,r0.xyz);break;case Pe:X0=max(n0.xyz,r0.xyz);break;case Qe:{x1.xyz=clamp(x1.xyz,R0(.0),x1.www);A Ib=clamp(1.-n0,R0(.0),R0(1.))*x1.w;X0=mix(min(R0(1.),x1.xyz/Ib),sign(x1.xyz),equal(Ib,R0(.0)));break;}case Se:{n0=clamp(n0,R0(.0),R0(1.));x1.xyz=clamp(x1.xyz,R0(.0),x1.www);if(x1.w==.0)x1.w=1.;A Jb=x1.w-x1.xyz;X0=1.-mix(min(R0(1.),Jb/(n0*x1.w)),sign(Jb),equal(n0,R0(.0)));break;}case Te:{A I6=n0*r0;X0=2.0*mix(I6,n0+r0-I6-0.5,greaterThan(n0,R0(0.5)));break;}case Ue:{for(int G0=0;G0<3;++G0){if(n0[G0]<=0.5)X0[G0]=(1.0-r0[G0]);else if(r0[G0]<=0.25)X0[G0]=((16.0*r0[G0]-12.0)*r0[G0]+3.0);else X0[G0]=(inversesqrt(r0[G0])-1.0);}X0=r0+r0*(2.0*n0-1.0)*X0;break;}case Ve:X0=abs(r0.xyz-n0.xyz);break;case We:X0=n0.xyz+r0.xyz-2.*n0.xyz*r0.xyz;break;
#ifdef EC
case Xe:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=Gb(n0.xyz,r0.xyz,r0.xyz);}break;case Ye:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=Gb(r0.xyz,n0.xyz,r0.xyz);}break;case Ze:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=m9(n0.xyz,r0.xyz);}break;case af:if(EC){n0.xyz=clamp(n0.xyz,R0(.0),R0(1.));X0=m9(r0.xyz,n0.xyz);}break;
#endif
}return X0;}e A U4(A n0,i x1,L r9){A X0=Ke(n0,x1,r9);return mix(n0,X0,R0(x1.w));}
#endif
#endif
