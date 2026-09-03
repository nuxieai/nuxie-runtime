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
c zb(A G1){return dot(G1,Q0(.30,.59,.11));}A k9(A Ab,A l9){c m9=zb(l9);A n9=Ab-zb(Ab);E Bb=B2(m9,1.0-m9)/max(B2(o9),B2(-h3(n9),M5(n9)));c se=min(G0(1.0),min(Bb.x,Bb.y));return n9*se+m9;}A Cb(A Q7,A Db,A l9){float te=M5(Db)-h3(Db);Q7-=h3(Q7);float ue=M5(Q7);float C2=te/max(o9,ue);return k9(Q7*C2,l9);}
#endif
A ve(A n0,i x1,N p9){A q0=F6(x1);A X0;switch(p9){case we:X0=n0.xyz*q0.xyz;break;case xe:X0=n0.xyz+q0.xyz-n0.xyz*q0.xyz;break;case ye:{A G6=n0*q0;X0=2.0*mix(G6,n0+q0-G6-0.5,greaterThan(q0,Q0(0.5)));break;}case ze:X0=min(n0.xyz,q0.xyz);break;case Ae:X0=max(n0.xyz,q0.xyz);break;case Be:{x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);A Eb=clamp(1.-n0,Q0(.0),Q0(1.))*x1.w;X0=mix(min(Q0(1.),x1.xyz/Eb),sign(x1.xyz),equal(Eb,Q0(.0)));break;}case De:{n0=clamp(n0,Q0(.0),Q0(1.));x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);if(x1.w==.0)x1.w=1.;A Fb=x1.w-x1.xyz;X0=1.-mix(min(Q0(1.),Fb/(n0*x1.w)),sign(Fb),equal(n0,Q0(.0)));break;}case Ee:{A G6=n0*q0;X0=2.0*mix(G6,n0+q0-G6-0.5,greaterThan(n0,Q0(0.5)));break;}case Fe:{for(int F0=0;F0<3;++F0){if(n0[F0]<=0.5)X0[F0]=(1.0-q0[F0]);else if(q0[F0]<=0.25)X0[F0]=((16.0*q0[F0]-12.0)*q0[F0]+3.0);else X0[F0]=(inversesqrt(q0[F0])-1.0);}X0=q0+q0*(2.0*n0-1.0)*X0;break;}case Ge:X0=abs(q0.xyz-n0.xyz);break;case He:X0=n0.xyz+q0.xyz-2.*n0.xyz*q0.xyz;break;
#ifdef GC
case Ie:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Cb(n0.xyz,q0.xyz,q0.xyz);}break;case Je:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=Cb(q0.xyz,n0.xyz,q0.xyz);}break;case Ke:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(n0.xyz,q0.xyz);}break;case Le:if(GC){n0.xyz=clamp(n0.xyz,Q0(.0),Q0(1.));X0=k9(q0.xyz,n0.xyz);}break;
#endif
}return X0;}e A T4(A n0,i x1,N p9){A X0=ve(n0,x1,p9);return mix(n0,X0,Q0(x1.w));}
#endif
#endif
