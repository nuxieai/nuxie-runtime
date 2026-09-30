#ifdef FRAGMENT
L1 z0(G2,l0);k1(V2,i0);z0(i6,o4);k1(L6,I7);M1 O1(HB){r(X1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(C2,R);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(j1,d);
#else
r(M,B2);
#endif
r(D0,d);
#ifdef ENABLE_CLIPPING
r(Y1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(N0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(g1,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
z2;
#endif
E U4=unpackHalf2x16(Z0(I7));d m9=U4.y;d r0=m9==D0?U4.x:J0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
r0+=j1;h2(I7);
#else
r0=ri(r0,M e1);d1(I7,packHalf2x16(D2(r0,D0)));
#endif
d o;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){o=ja(r0,J0(.0),J0(1.));}else
#endif
{o=abs(r0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&D0<.0){o=1.-J0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,J0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&Y1.x<.0){d l1=-Y1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d L5=Y1.y;if(L5!=.0){E P0=unpackHalf2x16(Z0(i0));d G6=P0.y;d v4;if(G6!=l1){v4=G6==L5?P0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
A0(o4,E0(v4,.0,.0,.0));
#endif
}else{v4=K0(o4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
y2(o4);
#endif
}o=min(o,v4);}}
#endif
d1(i0,packHalf2x16(D2(o,l1)));y2(l0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d l1=Y1.x;if(l1!=.0){E P0=unpackHalf2x16(Z0(i0));d G6=P0.y;o=(G6==l1)?min(P0.x,o):J0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d a5=k3(f5(N0));o=clamp(a5,J0(.0),o);}
#endif
i k=N7(
#ifdef ENABLE_MODULATED_IMAGE
C2,
#endif
#ifdef ENABLE_ADVANCED_BLEND
e3(g1),
#endif
X1 W2);i N1;if(m9!=D0){N1=K0(l0);
#ifndef DRAW_INTERIOR_TRIANGLES
A0(o4,N1);
#endif
}else{N1=K0(o4);
#ifndef DRAW_INTERIOR_TRIANGLES
y2(o4);
#endif
}
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&g1!=d6(A4)){k.xyz=X4(k.xyz,N1,e3(g1))*k.w;}
#endif
k*=o;
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){k=o3(k);}
#endif
d h3=k.w;k+=N1*(1.-h3);k.xyz=J2(k.xyz,h3,d0.xy,j.F3,j.G3);A0(l0,k);h2(i0);}
#if!defined(DRAW_INTERIOR_TRIANGLES)
A2;
#endif
d2;}
#endif
