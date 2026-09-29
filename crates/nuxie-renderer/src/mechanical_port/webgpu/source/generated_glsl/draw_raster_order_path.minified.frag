#ifdef FRAGMENT
J1 x0(S2,j0);j1(T2,g0);x0(g6,l4);j1(J6,H7);K1 M1(IB){r(f1,g);
#ifdef ENABLE_MODULATED_IMAGE
r(A2,Q);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);
#else
r(L,z2);
#endif
r(B0,c);
#ifdef ENABLE_CLIPPING
r(V1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(f2,c);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
x2;
#endif
E R4=unpackHalf2x16(Y0(H7));c i9=R4.y;c p0=i9==B0?R4.x:G0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
p0+=i1;e2(H7);
#else
p0=Yh(p0,L d1);c1(H7,packHalf2x16(B2(p0,B0)));
#endif
c n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=ca(p0,G0(.0),G0(1.));}else
#endif
{n=abs(p0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&B0<.0){n=1.-G0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,G0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&V1.x<.0){c k1=-V1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){c I5=V1.y;if(I5!=.0){E N0=unpackHalf2x16(Y0(g0));c E6=N0.y;c p4;if(E6!=k1){p4=E6==I5?N0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
y0(l4,C0(p4,.0,.0,.0));
#endif
}else{p4=I0(l4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
w2(l4);
#endif
}n=min(n,p4);}}
#endif
c1(g0,packHalf2x16(B2(n,k1)));w2(j0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){c k1=V1.x;if(k1!=.0){E N0=unpackHalf2x16(Y0(g0));c E6=N0.y;n=(E6==k1)?min(N0.x,n):G0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c Y4=h3(c5(M0));n=clamp(Y4,G0(.0),n);}
#endif
i j=M7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
n U2);i L1;if(i9!=B0){L1=I0(j0);
#ifndef DRAW_INTERIOR_TRIANGLES
y0(l4,L1);
#endif
}else{L1=I0(l4);
#ifndef DRAW_INTERIOR_TRIANGLES
w2(l4);
#endif
}
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(f2!=Z5(Q5)){j.xyz=U4(j.xyz,L1,a6(f2));}j.xyz*=j.w;}
#endif
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=l3(j);}
#endif
c v2=j.w;j+=L1*(1.-v2);j.xyz=F2(j.xyz,v2,a0.xy,m.A3,m.B3);y0(j0,j);e2(g0);}
#if!defined(DRAW_INTERIOR_TRIANGLES)
y2;
#endif
Z1;}
#endif
