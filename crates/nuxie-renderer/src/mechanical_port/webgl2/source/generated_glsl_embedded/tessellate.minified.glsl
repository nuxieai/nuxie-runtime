#define Rh 10
#ifdef DB
f1(f0)J(0,f,MD);J(1,f,ND);J(2,f,VC);
#ifdef Z9
J(3,uint,IE);J(4,uint,JE);J(5,uint,KE);J(6,uint,LE);
#else
J(3,X,TB);
#endif
g1
#endif
p2 H0 V(0,f,B6);H0 V(1,f,C6);H0 V(2,f,O4);H0 V(3,Q,P4);S2 V(4,uint,I7);h2
#ifdef DB
U3 j6(e3,h7,YC);V3 c4(h7,ea)B4 J4(Wc,Bg,PB);J4(Xc,Cg,ID);C4 y1(FG,f0,F,B,v){K(v,F,MD,f);K(v,F,ND,f);K(v,F,VC,f);
#ifdef Z9
K(v,F,IE,uint);K(v,F,JE,uint);K(v,F,KE,uint);K(v,F,LE,uint);X TB=X(IE,JE,KE,LE);
#else
K(v,F,TB,X);
#endif
T(B6,f);T(C6,f);T(O4,f);T(P4,Q);T(I7,uint);c v0=MD.xy;c A0=MD.zw;c E0=ND.xy;c L0=ND.zw;bool ee=B<4;float y=ee?VC.z:VC.w;int jb=int(ee?TB.x:TB.y);
#ifdef yc
int fe=jb<<16;if(TB.z==0xffffffffu){--fe;}float c9=float(fe>>16);
#else
float c9=float(jb<<16>>16);
#endif
float d9=float(jb>>16);c q2=c((B&1)==0?c9:d9,(B&2)==0?y+1.:y);if((d9-c9)*n.Ad<.0){q2.y=2.*y+1.-q2.y;}uint R2=TB.z&0x3ffu;uint ge=(TB.z>>10)&0x3ffu;uint m2=TB.z>>20;uint i0=TB.w;uint G8=i0&Sc;uint m0=G8>0u?K0(ID,max(G8,1u)-1u).z:0u;X L4=m0!=0u?K0(PB,m0*4u+1u):X(0u,0u,0u,0u);float L2=uintBitsToFloat(L4.z);float M2=uintBitsToFloat(L4.w);if(M2!=.0&&L2==.0){float he;float Sh=zf(v0,A0,E0,L0,he);float kb=M2*(1./sa);float Th=uf(v0,A0,E0,L0,he,kb);float J7=1.-Th*(1./D3);float Uh=dot(L0-v0,L0-v0)/(kb*kb);float Vh=(Uh-1.)*.5;J7=min(J7,Vh);J7=min(J7,.99);float Wh=.5*J7;float x=xc(Wh)*-2.+1.;float ie=l8(x*M2,Sh);f je=mix(v0.xyxy,L0.xyxy,f(1./3.,1./3.,2./3.,2./3.));A0=mix(A0,je.xy,ie);E0=mix(E0,je.zw,ie);}if((i0&Xf)!=0u){d0 U8=I1(uintBitsToFloat(K0(PB,m0*4u)));c ke=N0(U8,-2.*A0+E0+v0);c le=N0(U8,-2.*E0+L0+A0);float k1=max(dot(ke,ke),dot(le,le));float O3=max(ceil(sqrt(.75*4.*sqrt(k1))),1.);R2=min(uint(O3),R2);}uint e9=R2+ge+m2-1u;d0 J2=W9(v0,A0,E0,L0);float e1=acos(V9(J2[0],J2[1]));float o4=e1/float(ge);float lb=determinant(d0(E0-v0,L0-A0));if(lb==.0)lb=determinant(J2);if(lb<.0)o4=-o4;B6=f(v0,A0);C6=f(E0,L0);O4=f(float(e9)-abs(d9-q2.x),float(e9),(m2<<10)|R2,o4);P4.xy=VC.xy;if(m2>1u){d0 mb=d0(J2[1],VC.xy);float Xh=acos(V9(mb[0],mb[1]));float me=float(m2);if((i0&(a4|A8))==(y8|A8)){me-=2.;}float nb=Xh/me;if(determinant(mb)<.0)nb=-nb;P4.z=nb;}if(d9<c9){i0|=G3;}I7=i0;f W=p8(q2,2./Uf,n.Ad);
#ifdef SC
W.y=-W.y;
#endif
a0(B6);a0(C6);a0(O4);a0(P4);a0(I7);z1(W);}
#endif
#ifdef FB
E3 F3 d3(D4,GG){r(B6,f);r(C6,f);r(O4,f);r(P4,Q);r(I7,uint);c v0=B6.xy;c A0=B6.zw;c E0=C6.xy;c L0=C6.zw;d0 J2=W9(v0,A0,E0,L0);float Yh=max(floor(O4.x),.0);float e9=O4.y;uint ne=uint(O4.z);float R2=float(ne&0x3ffu);float m2=float(ne>>10);float o4=O4.w;uint i0=I7;float Q4=e9-m2;float U1=Yh;if(U1<=Q4){i0&=~a4;}else{v0=A0=E0=L0;J2=d0(J2[1],P4.xy);R2=1.;U1-=Q4;Q4=m2;o4=P4.z;if((i0&a4)>y8){if(U1<2.5)i0|=ta;if(U1>1.5&&U1<3.5)i0|=Qc;}else if((i0&A8)!=0u||(i0&a4)==z8){Q4-=2.;--U1;}i0|=o4<.0?B8:Rc;}c E5;float e1=.0;if(U1==.0||U1==Q4||(i0&a4)>y8){bool J8=U1<Q4*.5;E5=J8?v0:L0;e1=Ac(J8?J2[0]:J2[1]);}else if((i0&Pc)!=0u){E5=v0;if(U1>=float(qa/2u))E5=A0;if(U1>=float(qa*3u/4u))E5=E0;if(U1>=float(qa*7u/8u))E5=P4.xy;}else{float q1,F5;if(R2==Q4){q1=U1/R2;F5=.0;}else{c C,H,k2=A0-v0;c O6=L0-v0;c i8=E0-A0;H=i8-k2;C=-3.*i8+O6;c Zh=H*(R2*2.);c Q6=k2*(R2*R2);float f9=.0;float ai=min(R2-1.,U1);c ob=normalize(J2[0]);float bi=-abs(o4);float ci=(1.+U1)*abs(o4);for(int pb=Rh-1;pb>=0;--pb){float K7=f9+exp2(float(pb));if(K7<=ai){c qb=K7*C+Zh;qb=K7*qb+Q6;float di=dot(normalize(qb),ob);float rb=K7*bi+ci;rb=min(rb,D3);if(di>=cos(rb))f9=K7;}}float ei=f9/R2;float oe=U1-f9;float g9=acos(clamp(ob.x,-1.,1.));g9=ob.y>=.0?g9:-g9;e1=oe*o4+g9;c a3=c(sin(e1),-cos(e1));float m=dot(a3,C),h9=dot(a3,H),G1=dot(a3,k2);float fi=max(h9*h9-m*G1,.0);float w2=sqrt(fi);if(h9>.0)w2=-w2;w2-=h9;float pe=-.5*w2*m;c sb=(abs(w2*w2+pe)<abs(m*G1+pe))?c(w2,m):c(G1,w2);F5=(sb.y!=.0)?sb.x/sb.y:.0;F5=clamp(F5,.0,1.);if(oe==.0)F5=.0;q1=max(ei,F5);}c gi=d6(v0,A0,q1);c qe=d6(A0,E0,q1);c hi=d6(E0,L0,q1);c re=d6(gi,qe,q1);c se=d6(qe,hi,q1);E5=d6(re,se,q1);if(q1!=F5)e1=Ac(se-re);}D4 L7;L7.xy=ca(E5);if((i0&a4)==z8){L7.z=da((uint(Q4)<<16)|uint(U1));}else{L7.z=ca(mod(e1,q8));}L7.w=da(i0);K2(L7);}
#endif
