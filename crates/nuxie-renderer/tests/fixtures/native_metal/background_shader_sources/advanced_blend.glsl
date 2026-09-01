#ifdef GB
#ifdef JE
layout(
#ifdef GC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef AB
#ifdef GC
c yb(A G1){return dot(G1,Q0(.30,.59,.11));}A k9(A zb,A l9){c m9=yb(l9);A n9=zb-yb(zb);E Ab=B2(m9,1.0-m9)/max(B2(o9),B2(-h3(n9),L5(n9)));c re=min(G0(1.0),min(Ab.x,Ab.y));return n9*re+m9;}A Bb(A P7,A Cb,A l9){float se=L5(Cb)-h3(Cb);P7-=h3(P7);float te=L5(P7);float C2=se/max(o9,te);return k9(P7*C2,l9);}
#endif
A ue(A n0,i x1,N p9){A q0=E6(x1);A X0;switch(p9){case ve:X0=n0.xyz*q0.xyz;break;case we:X0=n0.xyz+q0.xyz-n0.xyz*q0.xyz;break;case xe:{A F6=n0*q0;X0=2.0*mix(F6,n0+q0-F6-0.5,greaterThan(q0,Q0(0.5)));break;}case ye:X0=min(n0.xyz,q0.xyz);break;case ze:X0=max(n0.xyz,q0.xyz);break;case Ae:{x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);A Db=clamp(1.-n0,Q0(.0),Q0(1.))*x1.w;X0=mix(min(Q0(1.),x1.xyz/Db),sign(x1.xyz),equal(Db,Q0(.0)));break;}case Ce:{n0=clamp(n0,Q0(.0),Q0(1.));x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);if(x1.w==.0)x1.w=1.;A Eb=x1.w-x1.xyz;X0=1.-mix(min(Q0(1.),Eb/(n0*x1.w)),sign(Eb),equal(n0,Q0(.0)));break;}case De:{A F6=n0*q0;X0=2.0*mix(F6,n0+q0-F6-0.5,greaterThan(n0,Q0(0.5)));break;}case Ee:{for(int E0=0;E0<3;++E0){if(n0[E0]<=0.5)X0[E0]=(1.0-q0[E0]);else if(q0[E0]<=0.25)X0[E0]=((16.0*q0[E0]-12.0)*q0[E0]+3.0);else X0[E0]=(inversesqrt(q0[E0])-1.0);}X0=q0+q0*(2.0*n0-1.0)*X0;break;}case Fe:X0=abs(q0.xyz-n0.xyz);break;case Ge:X0=n0.xyz+q0.xyz-2.*n0.xyz*q0.xyz;break;
#ifdef GC
case He:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Bb(n0.xyz,q0.xyz,q0.xyz);}break;case Ie:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Bb(q0.xyz,n0.xyz,q0.xyz);}break;case Je:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(n0.xyz,q0.xyz);}break;case Ke:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(q0.xyz,n0.xyz);}break;
#endif
}return X0;}e A S4(A n0,i x1,N p9){A X0=ue(n0,x1,p9);return mix(n0,X0,Q0(x1.w));}
#endif
#endif
