#define Ah 10
#ifdef VERTEX
g1(e0)L(0,g,JD);L(1,g,KD);L(2,g,VC);
#ifdef O3
L(3,uint,FE);L(4,uint,GE);L(5,uint,HE);L(6,uint,IE);
#else
L(3,G,UB);
#endif
h1
#endif
m2 H0 X(0,g,B6);H0 X(1,g,C6);H0 X(2,g,O4);H0 X(3,R,P4);Q2 X(4,uint,I7);g2
#ifdef VERTEX
U3 j6(d3,h7,YC);V3 c4(h7,aa)C4 J4(Pc,mg,QB);J4(Qc,ng,FD);D4 z1(AG,e0,F,B,v){M(v,F,JD,g);M(v,F,KD,g);M(v,F,VC,g);
#ifdef O3
M(v,F,FE,uint);M(v,F,GE,uint);M(v,F,HE,uint);M(v,F,IE,uint);G UB=G(FE,GE,HE,IE);
#else
M(v,F,UB,G);
#endif
V(B6,g);V(C6,g);V(O4,g);V(P4,R);V(I7,uint);d r0=JD.xy;d z0=JD.zw;d D0=KD.xy;d K0=KD.zw;bool Td=B<4;float y=Td?VC.z:VC.w;int eb=int(Td?UB.x:UB.y);
#ifdef qc
int Ud=eb<<16;if(UB.z==0xffffffffu){--Ud;}float Z8=float(Ud>>16);
#else
float Z8=float(eb<<16>>16);
#endif
float a9=float(eb>>16);d n2=d((B&1)==0?Z8:a9,(B&2)==0?y+1.:y);if((a9-Z8)*m.sd<.0){n2.y=2.*y+1.-n2.y;}uint P2=UB.z&0x3ffu;uint Vd=(UB.z>>10)&0x3ffu;uint k2=UB.z>>20;uint i0=UB.w;uint F8=i0&Lc;uint l0=F8>0u?J0(FD,max(F8,1u)-1u).z:0u;G L4=l0!=0u?J0(QB,l0*4u+1u):G(0u,0u,0u,0u);float J2=uintBitsToFloat(L4.z);float K2=uintBitsToFloat(L4.w);if(K2!=.0&&J2==.0){float Wd;float Bh=jf(r0,z0,D0,K0,Wd);float fb=K2*(1./na);float Ch=df(r0,z0,D0,K0,Wd,fb);float J7=1.-Ch*(1./D3);float Dh=dot(K0-r0,K0-r0)/(fb*fb);float Eh=(Dh-1.)*.5;J7=min(J7,Eh);J7=min(J7,.99);float Fh=.5*J7;float x=pc(Fh)*-2.+1.;float Xd=k8(x*K2,Bh);g Yd=mix(r0.xyxy,K0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Yd.xy,Xd);D0=mix(D0,Yd.zw,Xd);}if((i0&Hf)!=0u){f0 Zd=h2(uintBitsToFloat(J0(QB,l0*4u)));d ae=R0(Zd,-2.*z0+D0+r0);d be=R0(Zd,-2.*D0+K0+z0);float l1=max(dot(ae,ae),dot(be,be));float P3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);P2=min(uint(P3),P2);}uint c9=P2+Vd+k2-1u;f0 H2=T9(r0,z0,D0,K0);float e1=acos(S9(H2[0],H2[1]));float n4=e1/float(Vd);float gb=determinant(f0(D0-r0,K0-z0));if(gb==.0)gb=determinant(H2);if(gb<.0)n4=-n4;B6=g(r0,z0);C6=g(D0,K0);O4=g(float(c9)-abs(a9-n2.x),float(c9),(k2<<10)|P2,n4);P4.xy=VC.xy;if(k2>1u){f0 hb=f0(H2[1],VC.xy);float Gh=acos(S9(hb[0],hb[1]));float ce=float(k2);if((i0&(a4|z8))==(x8|z8)){ce-=2.;}float ib=Gh/ce;if(determinant(hb)<.0)ib=-ib;P4.z=ib;}if(a9<Z8){i0|=G3;}I7=i0;g W=o8(n2,2./Ef,m.sd);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
c0(B6);c0(C6);c0(O4);c0(P4);c0(I7);A1(W);}
#endif
#ifdef FRAGMENT
E3 F3 a3(E4,BG){r(B6,g);r(C6,g);r(O4,g);r(P4,R);r(I7,uint);d r0=B6.xy;d z0=B6.zw;d D0=C6.xy;d K0=C6.zw;f0 H2=T9(r0,z0,D0,K0);float Hh=max(floor(O4.x),.0);float c9=O4.y;uint de=uint(O4.z);float P2=float(de&0x3ffu);float k2=float(de>>10);float n4=O4.w;uint i0=I7;float Q4=c9-k2;float U1=Hh;if(U1<=Q4){i0&=~a4;}else{r0=z0=D0=K0;H2=f0(H2[1],P4.xy);P2=1.;U1-=Q4;Q4=k2;n4=P4.z;if((i0&a4)>x8){if(U1<2.5)i0|=oa;if(U1>1.5&&U1<3.5)i0|=Jc;}else if((i0&z8)!=0u||(i0&a4)==y8){Q4-=2.;--U1;}i0|=n4<.0?A8:Kc;}d F5;float e1=.0;if(U1==.0||U1==Q4||(i0&a4)>x8){bool I8=U1<Q4*.5;F5=I8?r0:K0;e1=sc(I8?H2[0]:H2[1]);}else if((i0&Ic)!=0u){F5=r0;if(U1>=float(la/2u))F5=z0;if(U1>=float(la*3u/4u))F5=D0;if(U1>=float(la*7u/8u))F5=P4.xy;}else{float r1,G5;if(P2==Q4){r1=U1/P2;G5=.0;}else{d C,H,i2=z0-r0;d O6=K0-r0;d h8=D0-z0;H=h8-i2;C=-3.*h8+O6;d Ih=H*(P2*2.);d Q6=i2*(P2*P2);float d9=.0;float Jh=min(P2-1.,U1);d jb=normalize(H2[0]);float Kh=-abs(n4);float Lh=(1.+U1)*abs(n4);for(int kb=Ah-1;kb>=0;--kb){float K7=d9+exp2(float(kb));if(K7<=Jh){d lb=K7*C+Ih;lb=K7*lb+Q6;float Mh=dot(normalize(lb),jb);float mb=K7*Kh+Lh;mb=min(mb,D3);if(Mh>=cos(mb))d9=K7;}}float Nh=d9/P2;float ee=U1-d9;float e9=acos(clamp(jb.x,-1.,1.));e9=jb.y>=.0?e9:-e9;e1=ee*n4+e9;d Y2=d(sin(e1),-cos(e1));float o=dot(Y2,C),f9=dot(Y2,H),H1=dot(Y2,i2);float Oh=max(f9*f9-o*H1,.0);float r2=sqrt(Oh);if(f9>.0)r2=-r2;r2-=f9;float fe=-.5*r2*o;d nb=(abs(r2*r2+fe)<abs(o*H1+fe))?d(r2,o):d(H1,r2);G5=(nb.y!=.0)?nb.x/nb.y:.0;G5=clamp(G5,.0,1.);if(ee==.0)G5=.0;r1=max(Nh,G5);}d Ph=d6(r0,z0,r1);d ge=d6(z0,D0,r1);d Qh=d6(D0,K0,r1);d he=d6(Ph,ge,r1);d ie=d6(ge,Qh,r1);F5=d6(he,ie,r1);if(r1!=G5)e1=sc(ie-he);}E4 L7;L7.xy=Y9(F5);if((i0&a4)==y8){L7.z=Z9((uint(Q4)<<16)|uint(U1));}else{L7.z=Y9(mod(e1,p8));}L7.w=Z9(i0);I2(L7);}
#endif
