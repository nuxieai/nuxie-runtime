#ifdef FRAGMENT
J1 y0(F2,k0);i1(U2,h0);y0(g6,m4);i1(K6,H7);K1 M1(IB){r(V1,f);
#ifdef ENABLE_MODULATED_IMAGE
r(B2,Q);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(h1,d);
#else
r(M,A2);
#endif
r(C0,d);
#ifdef ENABLE_CLIPPING
r(W1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,f);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(g2,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
y2;
#endif
E R4=unpackHalf2x16(Y0(H7));d l9=R4.y;d p0=l9==C0?R4.x:I0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
p0+=h1;f2(H7);
#else
p0=qi(p0,M d1);c1(H7,packHalf2x16(C2(p0,C0)));
#endif
d o;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){o=ia(p0,I0(.0),I0(1.));}else
#endif
{o=abs(p0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&C0<.0){o=1.-I0(abs(fract(o*.5)*2.+-1.));}
#endif
o=min(o,I0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&W1.x<.0){d j1=-W1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d H5=W1.y;if(H5!=.0){E O0=unpackHalf2x16(Y0(h0));d F6=O0.y;d q4;if(F6!=j1){q4=F6==H5?O0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
z0(m4,D0(q4,.0,.0,.0));
#endif
}else{q4=J0(m4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
x2(m4);
#endif
}o=min(o,q4);}}
#endif
c1(h0,packHalf2x16(C2(o,j1)));x2(k0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d j1=W1.x;if(j1!=.0){E O0=unpackHalf2x16(Y0(h0));d F6=O0.y;o=(F6==j1)?min(O0.x,o):I0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d X4=i3(a5(M0));o=clamp(X4,I0(.0),o);}
#endif
i j=M7(V1,
#ifdef ENABLE_MODULATED_IMAGE
B2,
#endif
o V2);i L1;if(l9!=C0){L1=J0(k0);
#ifndef DRAW_INTERIOR_TRIANGLES
z0(m4,L1);
#endif
}else{L1=J0(m4);
#ifndef DRAW_INTERIOR_TRIANGLES
x2(m4);
#endif
}
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(g2!=Z5(Q5)){j.xyz=U4(j.xyz,L1,a6(g2));}j.xyz*=j.w;}
#endif
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=l3(j);}
#endif
d f3=j.w;j+=L1*(1.-f3);j.xyz=I2(j.xyz,f3,c0.xy,l.C3,l.D3);z0(k0,j);f2(h0);}
#if!defined(DRAW_INTERIOR_TRIANGLES)
z2;
#endif
a2;}
#endif
