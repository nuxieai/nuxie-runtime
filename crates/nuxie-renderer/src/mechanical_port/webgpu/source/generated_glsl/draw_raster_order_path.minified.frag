#ifdef FRAGMENT
V1 C0(U2,n0);q1(i3,m0);C0(w6,G4);q1(d7,j8);W1 Y1(IB){q(O0,e);
#ifdef ENABLE_MODULATED_IMAGE
q(V0,M);
#endif
#ifdef DRAW_INTERIOR_TRIANGLES
q(o1,d);
#else
q(S,Q2);
#endif
q(G0,d);
#ifdef ENABLE_CLIPPING
q(j2,D);
#endif
#ifdef ENABLE_CLIP_RECT
q(W0,e);
#endif
#ifdef ENABLE_ADVANCED_BLEND
q(P0,d);
#endif
#if!defined(DRAW_INTERIOR_TRIANGLES)
O2;
#endif
D k5=unpackHalf2x16(l1(j8));d ba=k5.y;d w0=ba==G0?k5.x:I0(.0);
#ifdef DRAW_INTERIOR_TRIANGLES
w0+=o1;h2(j8);
#else
w0=Hj(w0,S n1);m1(j8,packHalf2x16(R2(w0,G0)));
#endif
d l;
#ifdef CLOCKWISE_FILL
if(CLOCKWISE_FILL){l=Xa(w0,I0(.0),I0(1.));}else
#endif
{l=abs(w0);
#ifdef ENABLE_EVEN_ODD
if(ENABLE_EVEN_ODD&&G0<.0){l=1.-I0(abs(fract(l*.5)*2.+-1.));}
#endif
l=min(l,I0(1.));}
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING&&j2.x<.0){d z1=-j2.x;
#ifdef ENABLE_NESTED_CLIPPING
if(ENABLE_NESTED_CLIPPING){d a6=j2.y;if(a6!=.0){D X0=unpackHalf2x16(l1(m0));d Y6=X0.y;d K4;if(Y6!=z1){K4=Y6==a6?X0.x:.0;
#ifndef DRAW_INTERIOR_TRIANGLES
z0(G4,H0(K4,.0,.0,.0));
#endif
}else{K4=R0(G4).x;
#ifndef DRAW_INTERIOR_TRIANGLES
N2(G4);
#endif
}l=min(l,K4);}}
#endif
m1(m0,packHalf2x16(R2(l,z1)));N2(n0);}else
#endif
{
#ifdef ENABLE_CLIPPING
if(ENABLE_CLIPPING){d z1=j2.x;if(z1!=.0){D X0=unpackHalf2x16(l1(m0));d Y6=X0.y;l=(Y6==z1)?min(X0.x,l):I0(.0);}}
#endif
#ifdef ENABLE_CLIP_RECT
if(ENABLE_CLIP_RECT){d r5=A3(T4(W0));l=clamp(r5,I0(.0),l);}
#endif
i n=o8(
#ifdef ENABLE_MODULATED_IMAGE
V0,
#endif
#ifdef ENABLE_ADVANCED_BLEND
X2(P0),
#endif
O0 l3);i A1;if(ba!=G0){A1=R0(n0);
#ifndef DRAW_INTERIOR_TRIANGLES
z0(G4,A1);
#endif
}else{A1=R0(G4);
#ifndef DRAW_INTERIOR_TRIANGLES
N2(G4);
#endif
}bool Ef=false;
#ifdef ENABLE_MODULATED_IMAGE
Ef=ENABLE_MODULATED_IMAGE&&V0.z<.0;
#endif
if(Ef){
#ifdef ENABLE_MODULATED_IMAGE
uint Nj=uint(-V0.z-1.);d Oj=Gj(n,Nj);n=A1*mix(I0(1.),Oj,l);z0(n0,n);h2(m0);
#endif
}else{
#ifdef ENABLE_ADVANCED_BLEND
if(ENABLE_ADVANCED_BLEND&&P0!=D5(T3)){n.xyz=L4(n.xyz,A1,X2(P0))*n.w;}
#endif
n*=l;d w3=n.w;n+=A1*(1.-w3);n.xyz=I2(n.xyz,w3,d0.xy,j.E3,j.F3);z0(n0,n);h2(m0);}}
#if!defined(DRAW_INTERIOR_TRIANGLES)
P2;
#endif
p2;}
#endif
