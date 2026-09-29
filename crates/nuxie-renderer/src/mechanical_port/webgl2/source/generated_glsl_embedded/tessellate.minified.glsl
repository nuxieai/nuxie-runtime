#define zh 10
#ifdef DB
g1(e0)O(0,g,ID);O(1,g,JD);O(2,g,UC);
#ifdef U9
O(3,uint,EE);O(4,uint,FE);O(5,uint,GE);O(6,uint,HE);
#else
O(3,X,TB);
#endif
h1
#endif
m2 H0 W(0,g,A6);H0 W(1,g,B6);H0 W(2,g,O4);H0 W(3,Q,P4);Q2 W(4,uint,I7);g2
#ifdef DB
T3 i6(d3,h7,XC);U3 a4(h7,Z9)B4 J4(Pc,lg,PB);J4(Qc,mg,ED);C4 z1(AG,e0,F,B,A){P(A,F,ID,g);P(A,F,JD,g);P(A,F,UC,g);
#ifdef U9
P(A,F,EE,uint);P(A,F,FE,uint);P(A,F,GE,uint);P(A,F,HE,uint);X TB=X(EE,FE,GE,HE);
#else
P(A,F,TB,X);
#endif
U(A6,g);U(B6,g);U(O4,g);U(P4,Q);U(I7,uint);d r0=ID.xy;d z0=ID.zw;d D0=JD.xy;d K0=JD.zw;bool Td=B<4;float y=Td?UC.z:UC.w;int fb=int(Td?TB.x:TB.y);
#ifdef rc
int Ud=fb<<16;if(TB.z==0xffffffffu){--Ud;}float Z8=float(Ud>>16);
#else
float Z8=float(fb<<16>>16);
#endif
float a9=float(fb>>16);d n2=d((B&1)==0?Z8:a9,(B&2)==0?y+1.:y);if((a9-Z8)*m.sd<.0){n2.y=2.*y+1.-n2.y;}uint P2=TB.z&0x3ffu;uint Vd=(TB.z>>10)&0x3ffu;uint k2=TB.z>>20;uint h0=TB.w;uint F8=h0&Lc;uint l0=F8>0u?J0(ED,max(F8,1u)-1u).z:0u;X L4=l0!=0u?J0(PB,l0*4u+1u):X(0u,0u,0u,0u);float J2=uintBitsToFloat(L4.z);float K2=uintBitsToFloat(L4.w);if(K2!=.0&&J2==.0){float Wd;float Ah=jf(r0,z0,D0,K0,Wd);float gb=K2*(1./na);float Bh=df(r0,z0,D0,K0,Wd,gb);float J7=1.-Bh*(1./C3);float Ch=dot(K0-r0,K0-r0)/(gb*gb);float Dh=(Ch-1.)*.5;J7=min(J7,Dh);J7=min(J7,.99);float Eh=.5*J7;float x=qc(Eh)*-2.+1.;float Xd=k8(x*K2,Ah);g Yd=mix(r0.xyxy,K0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Yd.xy,Xd);D0=mix(D0,Yd.zw,Xd);}if((h0&Hf)!=0u){f0 Zd=h2(uintBitsToFloat(J0(PB,l0*4u)));d ae=R0(Zd,-2.*z0+D0+r0);d be=R0(Zd,-2.*D0+K0+z0);float l1=max(dot(ae,ae),dot(be,be));float N3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);P2=min(uint(N3),P2);}uint c9=P2+Vd+k2-1u;f0 H2=R9(r0,z0,D0,K0);float e1=acos(Q9(H2[0],H2[1]));float n4=e1/float(Vd);float hb=determinant(f0(D0-r0,K0-z0));if(hb==.0)hb=determinant(H2);if(hb<.0)n4=-n4;A6=g(r0,z0);B6=g(D0,K0);O4=g(float(c9)-abs(a9-n2.x),float(c9),(k2<<10)|P2,n4);P4.xy=UC.xy;if(k2>1u){f0 ib=f0(H2[1],UC.xy);float Fh=acos(Q9(ib[0],ib[1]));float ce=float(k2);if((h0&(Z3|z8))==(x8|z8)){ce-=2.;}float jb=Fh/ce;if(determinant(ib)<.0)jb=-jb;P4.z=jb;}if(a9<Z8){h0|=F3;}I7=h0;g V=o8(n2,2./Ef,m.sd);
#ifdef RC
V.y=-V.y;
#endif
c0(A6);c0(B6);c0(O4);c0(P4);c0(I7);A1(V);}
#endif
#ifdef GB
D3 E3 a3(D4,BG){r(A6,g);r(B6,g);r(O4,g);r(P4,Q);r(I7,uint);d r0=A6.xy;d z0=A6.zw;d D0=B6.xy;d K0=B6.zw;f0 H2=R9(r0,z0,D0,K0);float Gh=max(floor(O4.x),.0);float c9=O4.y;uint de=uint(O4.z);float P2=float(de&0x3ffu);float k2=float(de>>10);float n4=O4.w;uint h0=I7;float Q4=c9-k2;float U1=Gh;if(U1<=Q4){h0&=~Z3;}else{r0=z0=D0=K0;H2=f0(H2[1],P4.xy);P2=1.;U1-=Q4;Q4=k2;n4=P4.z;if((h0&Z3)>x8){if(U1<2.5)h0|=oa;if(U1>1.5&&U1<3.5)h0|=Jc;}else if((h0&z8)!=0u||(h0&Z3)==y8){Q4-=2.;--U1;}h0|=n4<.0?A8:Kc;}d E5;float e1=.0;if(U1==.0||U1==Q4||(h0&Z3)>x8){bool I8=U1<Q4*.5;E5=I8?r0:K0;e1=tc(I8?H2[0]:H2[1]);}else if((h0&Ic)!=0u){E5=r0;if(U1>=float(la/2u))E5=z0;if(U1>=float(la*3u/4u))E5=D0;if(U1>=float(la*7u/8u))E5=P4.xy;}else{float r1,F5;if(P2==Q4){r1=U1/P2;F5=.0;}else{d C,H,i2=z0-r0;d O6=K0-r0;d h8=D0-z0;H=h8-i2;C=-3.*h8+O6;d Hh=H*(P2*2.);d Q6=i2*(P2*P2);float d9=.0;float Ih=min(P2-1.,U1);d kb=normalize(H2[0]);float Jh=-abs(n4);float Kh=(1.+U1)*abs(n4);for(int lb=zh-1;lb>=0;--lb){float K7=d9+exp2(float(lb));if(K7<=Ih){d mb=K7*C+Hh;mb=K7*mb+Q6;float Lh=dot(normalize(mb),kb);float nb=K7*Jh+Kh;nb=min(nb,C3);if(Lh>=cos(nb))d9=K7;}}float Mh=d9/P2;float ee=U1-d9;float e9=acos(clamp(kb.x,-1.,1.));e9=kb.y>=.0?e9:-e9;e1=ee*n4+e9;d Y2=d(sin(e1),-cos(e1));float o=dot(Y2,C),f9=dot(Y2,H),H1=dot(Y2,i2);float Nh=max(f9*f9-o*H1,.0);float r2=sqrt(Nh);if(f9>.0)r2=-r2;r2-=f9;float fe=-.5*r2*o;d ob=(abs(r2*r2+fe)<abs(o*H1+fe))?d(r2,o):d(H1,r2);F5=(ob.y!=.0)?ob.x/ob.y:.0;F5=clamp(F5,.0,1.);if(ee==.0)F5=.0;r1=max(Mh,F5);}d Oh=c6(r0,z0,r1);d ge=c6(z0,D0,r1);d Ph=c6(D0,K0,r1);d he=c6(Oh,ge,r1);d ie=c6(ge,Ph,r1);E5=c6(he,ie,r1);if(r1!=F5)e1=tc(ie-he);}D4 L7;L7.xy=X9(E5);if((h0&Z3)==y8){L7.z=Y9((uint(Q4)<<16)|uint(U1));}else{L7.z=X9(mod(e1,p8));}L7.w=Y9(h0);I2(L7);}
#endif
