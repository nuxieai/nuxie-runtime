#define Sh 10
#ifdef DB
f1(f0)J(0,f,MD);J(1,f,ND);J(2,f,VC);
#ifdef Z9
J(3,uint,IE);J(4,uint,JE);J(5,uint,KE);J(6,uint,LE);
#else
J(3,X,TB);
#endif
g1
#endif
p2 H0 V(0,f,C6);H0 V(1,f,D6);H0 V(2,f,O4);H0 V(3,Q,P4);T2 V(4,uint,I7);h2
#ifdef DB
V3 k6(e3,i7,YC);W3 d4(i7,ea)B4 J4(Xc,Cg,PB);J4(Yc,Dg,ID);C4 y1(GG,f0,F,B,v){K(v,F,MD,f);K(v,F,ND,f);K(v,F,VC,f);
#ifdef Z9
K(v,F,IE,uint);K(v,F,JE,uint);K(v,F,KE,uint);K(v,F,LE,uint);X TB=X(IE,JE,KE,LE);
#else
K(v,F,TB,X);
#endif
T(C6,f);T(D6,f);T(O4,f);T(P4,Q);T(I7,uint);c v0=MD.xy;c A0=MD.zw;c E0=ND.xy;c L0=ND.zw;bool fe=B<4;float y=fe?VC.z:VC.w;int jb=int(fe?TB.x:TB.y);
#ifdef zc
int ge=jb<<16;if(TB.z==0xffffffffu){--ge;}float c9=float(ge>>16);
#else
float c9=float(jb<<16>>16);
#endif
float d9=float(jb>>16);c q2=c((B&1)==0?c9:d9,(B&2)==0?y+1.:y);if((d9-c9)*l.Bd<.0){q2.y=2.*y+1.-q2.y;}uint S2=TB.z&0x3ffu;uint he=(TB.z>>10)&0x3ffu;uint m2=TB.z>>20;uint i0=TB.w;uint G8=i0&Tc;uint m0=G8>0u?K0(ID,max(G8,1u)-1u).z:0u;X L4=m0!=0u?K0(PB,m0*4u+1u):X(0u,0u,0u,0u);float M2=uintBitsToFloat(L4.z);float N2=uintBitsToFloat(L4.w);if(N2!=.0&&M2==.0){float ie;float Th=Af(v0,A0,E0,L0,ie);float kb=N2*(1./sa);float Uh=vf(v0,A0,E0,L0,ie,kb);float J7=1.-Uh*(1./E3);float Vh=dot(L0-v0,L0-v0)/(kb*kb);float Wh=(Vh-1.)*.5;J7=min(J7,Wh);J7=min(J7,.99);float Xh=.5*J7;float x=yc(Xh)*-2.+1.;float je=l8(x*N2,Th);f ke=mix(v0.xyxy,L0.xyxy,f(1./3.,1./3.,2./3.,2./3.));A0=mix(A0,ke.xy,je);E0=mix(E0,ke.zw,je);}if((i0&Yf)!=0u){d0 U8=I1(uintBitsToFloat(K0(PB,m0*4u)));c le=N0(U8,-2.*A0+E0+v0);c me=N0(U8,-2.*E0+L0+A0);float k1=max(dot(le,le),dot(me,me));float P3=max(ceil(sqrt(.75*4.*sqrt(k1))),1.);S2=min(uint(P3),S2);}uint e9=S2+he+m2-1u;d0 K2=W9(v0,A0,E0,L0);float e1=acos(V9(K2[0],K2[1]));float o4=e1/float(he);float lb=determinant(d0(E0-v0,L0-A0));if(lb==.0)lb=determinant(K2);if(lb<.0)o4=-o4;C6=f(v0,A0);D6=f(E0,L0);O4=f(float(e9)-abs(d9-q2.x),float(e9),(m2<<10)|S2,o4);P4.xy=VC.xy;if(m2>1u){d0 mb=d0(K2[1],VC.xy);float Yh=acos(V9(mb[0],mb[1]));float ne=float(m2);if((i0&(c4|A8))==(y8|A8)){ne-=2.;}float nb=Yh/ne;if(determinant(mb)<.0)nb=-nb;P4.z=nb;}if(d9<c9){i0|=H3;}I7=i0;f W=p8(q2,2./Vf,l.Bd);
#ifdef SC
W.y=-W.y;
#endif
a0(C6);a0(D6);a0(O4);a0(P4);a0(I7);z1(W);}
#endif
#ifdef FB
F3 G3 d3(D4,HG){r(C6,f);r(D6,f);r(O4,f);r(P4,Q);r(I7,uint);c v0=C6.xy;c A0=C6.zw;c E0=D6.xy;c L0=D6.zw;d0 K2=W9(v0,A0,E0,L0);float Zh=max(floor(O4.x),.0);float e9=O4.y;uint oe=uint(O4.z);float S2=float(oe&0x3ffu);float m2=float(oe>>10);float o4=O4.w;uint i0=I7;float Q4=e9-m2;float U1=Zh;if(U1<=Q4){i0&=~c4;}else{v0=A0=E0=L0;K2=d0(K2[1],P4.xy);S2=1.;U1-=Q4;Q4=m2;o4=P4.z;if((i0&c4)>y8){if(U1<2.5)i0|=ta;if(U1>1.5&&U1<3.5)i0|=Rc;}else if((i0&A8)!=0u||(i0&c4)==z8){Q4-=2.;--U1;}i0|=o4<.0?B8:Sc;}c F5;float e1=.0;if(U1==.0||U1==Q4||(i0&c4)>y8){bool J8=U1<Q4*.5;F5=J8?v0:L0;e1=Bc(J8?K2[0]:K2[1]);}else if((i0&Qc)!=0u){F5=v0;if(U1>=float(qa/2u))F5=A0;if(U1>=float(qa*3u/4u))F5=E0;if(U1>=float(qa*7u/8u))F5=P4.xy;}else{float q1,G5;if(S2==Q4){q1=U1/S2;G5=.0;}else{c C,H,k2=A0-v0;c P6=L0-v0;c i8=E0-A0;H=i8-k2;C=-3.*i8+P6;c ai=H*(S2*2.);c R6=k2*(S2*S2);float f9=.0;float bi=min(S2-1.,U1);c ob=normalize(K2[0]);float ci=-abs(o4);float di=(1.+U1)*abs(o4);for(int pb=Sh-1;pb>=0;--pb){float K7=f9+exp2(float(pb));if(K7<=bi){c qb=K7*C+ai;qb=K7*qb+R6;float ei=dot(normalize(qb),ob);float rb=K7*ci+di;rb=min(rb,E3);if(ei>=cos(rb))f9=K7;}}float fi=f9/S2;float pe=U1-f9;float g9=acos(clamp(ob.x,-1.,1.));g9=ob.y>=.0?g9:-g9;e1=pe*o4+g9;c a3=c(sin(e1),-cos(e1));float n=dot(a3,C),h9=dot(a3,H),G1=dot(a3,k2);float gi=max(h9*h9-n*G1,.0);float w2=sqrt(gi);if(h9>.0)w2=-w2;w2-=h9;float qe=-.5*w2*n;c sb=(abs(w2*w2+qe)<abs(n*G1+qe))?c(w2,n):c(G1,w2);G5=(sb.y!=.0)?sb.x/sb.y:.0;G5=clamp(G5,.0,1.);if(pe==.0)G5=.0;q1=max(fi,G5);}c hi=e6(v0,A0,q1);c re=e6(A0,E0,q1);c ii=e6(E0,L0,q1);c se=e6(hi,re,q1);c te=e6(re,ii,q1);F5=e6(se,te,q1);if(q1!=G5)e1=Bc(te-se);}D4 L7;L7.xy=ca(F5);if((i0&c4)==z8){L7.z=da((uint(Q4)<<16)|uint(U1));}else{L7.z=ca(mod(e1,q8));}L7.w=da(i0);L2(L7);}
#endif
