#ifdef FRAGMENT
Q1 A0(L2,o0);o1(d3,m0);A0(n6,B4);o1(T6,R7);R1 T1(IB){q(a1,e);
#ifdef ENABLE_MODULATED_IMAGE
q(F1,P);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(m1,d);
#else
q(S,H2);
#endif
q(F0,d);
#ifdef ENABLE_CLIPPING
q(l1,C);
#endif
#ifdef ENABLE_CLIP_RECT
q(R0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(Q0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
F2;
#endif
C d5=unpackHalf2x16(h1(R7));d C9=d5.y;d w0=C9==F0?d5.x:M0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=m1;k2(R7);
#else
w0=Ni(w0,S k1);j1(R7,packHalf2x16(I2(w0,F0)));
#endif
d o;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){o=Aa(w0,M0(.0),M0(1.));}else
#endif
{o=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&F0<.0){o=1.-M0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,M0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&l1.x<.0){d X0=-l1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d E4=l1.y;if(E4!=.0){C T0=unpackHalf2x16(h1(m0));d O6=T0.y;d G4;if(O6!=X0){G4=O6==E4?T0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
B0(B4,G0(G4,.0,.0,.0));
#endif
}else{G4=N0(B4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
E2(B4);
#endif
}o=min(o,G4);}}
#endif
j1(m0,packHalf2x16(I2(o,X0)));E2(o0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d X0=l1.x;if(X0!=.0){C T0=unpackHalf2x16(h1(m0));d O6=T0.y;o=(O6==X0)?min(T0.x,o):M0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d l5=v3(q5(R0));o=clamp(l5,M0(.0),o);}
#endif
i l=Y7(
#ifdef ENABLE_MODULATED_IMAGE
F1,
#endif
#ifdef ENABLE_ADVANCED_BLEND
k3(Q0),
#endif
a1 e3);i S1;if(C9!=F0){S1=N0(o0);
#ifndef DRAW_INTERIOR_TRIANGLES
B0(B4,S1);
#endif
}else{S1=N0(B4);
#ifndef DRAW_INTERIOR_TRIANGLES
E2(B4);
#endif
}
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&Q0!=i6(L4)){l.xyz=h5(l.xyz,S1,k3(Q0))*l.w;}
#endif
l*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){l=z3(l);}
#endif
d n3=l.w;l+=S1*(1.-n3);l.xyz=O2(l.xyz,n3,f0.xy,j.M3,j.N3);B0(o0,l);k2(m0);}
#if!defined(DRAW_INTERIOR_TRIANGLES)
G2;
#endif
g2;}
#endif
