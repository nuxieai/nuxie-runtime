#ifdef FRAGMENT
I1 w0(S2,j0);j1(T2,h0);w0(f6,k4);j1(I6,G7);J1 L1(JB){r(f1,g);
#ifdef ENABLE_MODULATED_IMAGE
r(A2,R);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
r(i1,c);
#else
r(O,z2);
#endif
r(B0,c);
#ifdef ENABLE_CLIPPING
r(U1,E);
#endif
#ifdef ENABLE_CLIP_RECT
r(M0,g);
#endif
#ifdef ENABLE_ADVANCED_BLEND
r(e2,c);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
w2;
#endif
E P4=unpackHalf2x16(Y0(G7));c i9=P4.y;c p0=i9==B0?P4.x:G0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
p0+=i1;d2(G7);
#else
p0=Rh(p0,O d1);c1(G7,packHalf2x16(B2(p0,B0)));
#endif
c n;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){n=da(p0,G0(.0),G0(1.));}else
#endif
{n=abs(p0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&B0<.0){n=1.-G0(abs(fract(n*.5)*2.+-1.));}
#endif
n=min(n,G0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&U1.x<.0){c k1=-U1.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){c H5=U1.y;if(H5!=.0){E N0=unpackHalf2x16(Y0(h0));c D6=N0.y;c o4;if(D6!=k1){o4=D6==H5?N0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
x0(k4,C0(o4,.0,.0,.0));
#endif
}else{o4=I0(k4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
v2(k4);
#endif
}n=min(n,o4);}}
#endif
c1(h0,packHalf2x16(B2(n,k1)));v2(j0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){c k1=U1.x;if(k1!=.0){E N0=unpackHalf2x16(Y0(h0));c D6=N0.y;n=(D6==k1)?min(N0.x,n):G0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){c W4=h3(Z4(M0));n=clamp(W4,G0(.0),n);}
#endif
i j=L7(f1,
#ifdef ENABLE_MODULATED_IMAGE
A2,
#endif
n U2);i K1;if(i9!=B0){K1=I0(j0);
#ifndef DRAW_INTERIOR_TRIANGLES
x0(k4,K1);
#endif
}else{K1=I0(k4);
#ifndef DRAW_INTERIOR_TRIANGLES
v2(k4);
#endif
}
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND){if(e2!=Y5(P5)){j.xyz=S4(j.xyz,K1,Z5(e2));}j.xyz*=j.w;}
#endif
#ifdef NEEDS_GAMMA_CORRECTION
if(NEEDS_GAMMA_CORRECTION){j=m3(j);}
#endif
c r2=j.w;j+=K1*(1.-r2);j.xyz=F2(j.xyz,r2,Z.xy,m.B3,m.C3);x0(j0,j);d2(h0);}
#if!defined(DRAW_INTERIOR_TRIANGLES)
x2;
#endif
Y1;}
#endif
