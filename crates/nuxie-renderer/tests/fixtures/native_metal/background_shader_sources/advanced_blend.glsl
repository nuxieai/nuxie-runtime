#ifdef GB
#ifdef GE
layout(
#ifdef FC
blend_support_all_equations
#else
blend_support_multiply,blend_support_screen,blend_support_overlay,blend_support_darken,blend_support_lighten,blend_support_colordodge,blend_support_colorburn,blend_support_hardlight,blend_support_softlight,blend_support_difference,blend_support_exclusion
#endif
)out;
#endif
#ifdef AB
#ifdef FC
c wb(v G1){return dot(G1,Q0(.30,.59,.11));}v k9(v xb,v l9){c m9=wb(l9);v n9=xb-wb(xb);E yb=A2(m9,1.0-m9)/max(A2(o9),A2(-f3(n9),J5(n9)));c oe=min(G0(1.0),min(yb.x,yb.y));return n9*oe+m9;}v zb(v N7,v Ab,v l9){float pe=J5(Ab)-f3(Ab);N7-=f3(N7);float qe=J5(N7);float B2=pe/max(o9,qe);return k9(N7*B2,l9);}
#endif
v re(v m0,i x1,N p9){v q0=C6(x1);v X0;switch(p9){case se:X0=m0.xyz*q0.xyz;break;case te:X0=m0.xyz+q0.xyz-m0.xyz*q0.xyz;break;case ue:{v D6=m0*q0;X0=2.0*mix(D6,m0+q0-D6-0.5,greaterThan(q0,Q0(0.5)));break;}case ve:X0=min(m0.xyz,q0.xyz);break;case we:X0=max(m0.xyz,q0.xyz);break;case xe:{x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);v Bb=clamp(1.-m0,Q0(.0),Q0(1.))*x1.w;X0=mix(min(Q0(1.),x1.xyz/Bb),sign(x1.xyz),equal(Bb,Q0(.0)));break;}case ze:{m0=clamp(m0,Q0(.0),Q0(1.));x1.xyz=clamp(x1.xyz,Q0(.0),x1.www);if(x1.w==.0)x1.w=1.;v Cb=x1.w-x1.xyz;X0=1.-mix(min(Q0(1.),Cb/(m0*x1.w)),sign(Cb),equal(m0,Q0(.0)));break;}case Ae:{v D6=m0*q0;X0=2.0*mix(D6,m0+q0-D6-0.5,greaterThan(m0,Q0(0.5)));break;}case Be:{for(int E0=0;E0<3;++E0){if(m0[E0]<=0.5)X0[E0]=(1.0-q0[E0]);else if(q0[E0]<=0.25)X0[E0]=((16.0*q0[E0]-12.0)*q0[E0]+3.0);else X0[E0]=(inversesqrt(q0[E0])-1.0);}X0=q0+q0*(2.0*m0-1.0)*X0;break;}case Ce:X0=abs(q0.xyz-m0.xyz);break;case De:X0=m0.xyz+q0.xyz-2.*m0.xyz*q0.xyz;break;
#ifdef FC
case Ee:if(FC){m0.xyz=clamp(m0.xyz,Q0(.0),Q0(1.));X0=zb(m0.xyz,q0.xyz,q0.xyz);}break;case Fe:if(FC){m0.xyz=clamp(m0.xyz,Q0(.0),Q0(1.));X0=zb(q0.xyz,m0.xyz,q0.xyz);}break;case Ge:if(FC){m0.xyz=clamp(m0.xyz,Q0(.0),Q0(1.));X0=k9(m0.xyz,q0.xyz);}break;case He:if(FC){m0.xyz=clamp(m0.xyz,Q0(.0),Q0(1.));X0=k9(q0.xyz,m0.xyz);}break;
#endif
}return X0;}e v Q4(v m0,i x1,N p9){v X0=re(m0,x1,p9);return mix(m0,X0,Q0(x1.w));}
#endif
#endif
