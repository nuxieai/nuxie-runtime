#define rj 10
#ifdef BB
e1(f0) K(0,f,PD);K(1,f,QD);K(2,f,UC);
#ifdef ab
K(3,uint,LE);K(4,uint,ME);K(5,uint,NE);K(6,uint,OE);
#else
K(3,O,WB);
#endif
f1
#endif
v2 F0 W(0,f,d7);F0 W(1,f,e7);F0 W(2,f,m5);F0 W(3,M,n5);h3 W(4,uint,q8);k2
#ifdef BB
q4 K6(r3,L7,AD);r4 a4(L7,db) Y4 j5(ce,gi,KB);j5(de,hi,DD);Z4 y1(KG,f0,B,F,r){L(r,B,PD,f);L(r,B,QD,f);L(r,B,UC,f);
#ifdef ab
L(r,B,LE,uint);L(r,B,ME,uint);L(r,B,NE,uint);L(r,B,OE,uint);O WB=O(LE,ME,NE,OE);
#else
L(r,B,WB,O);
#endif
V(d7,f);V(e7,f);V(m5,f);V(n5,M);V(q8,uint);c A0=PD.xy;c D0=PD.zw;c K0=QD.xy;c T0=QD.zw;bool ef=F<4;float y=ef?UC.z:UC.w;int wc=int(ef?WB.x:WB.y);
#ifdef eb
int ff=wc<<16;if(WB.z==0xffffffffu){--ff;}float ba=float(ff>>16);
#else
float ba=float(wc<<16>>16);
#endif
float ca=float(wc>>16);c F2=c((F&1)==0?ba:ca,(F&2)==0?y+1.:y);if((ca-ba)*j.Ee<.0){F2.y=2.*y+1.-F2.y;}uint g3=WB.z&0x3ffu;uint gf=(WB.z>>10)&0x3ffu;uint D2=WB.z>>20;uint a0=WB.w;uint D6=a0&xb;uint c0=D6>0u?p0(DD,max(D6,1u)-1u).z:0u;O L3=c0!=0u?p0(KB,c0*4u+1u):O(0u,0u,0u,0u);float C2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);if(Y2!=.0&&C2==.0){float hf;float sj=Eg(A0,D0,K0,T0,hf);float xc=Y2*(1./sb);float tj=zg(A0,D0,K0,T0,hf,xc);float r8=1.-tj*(1./p4);float uj=dot(T0-A0,T0-A0)/(xc*xc);float vj=(uj-1.)*.5;r8=min(r8,vj);r8=min(r8,.99);float wj=.5*r8;float x=Dd(wj)*-2.+1.;float jf=c9(x*Y2,sj);f kf=mix(A0.xyxy,T0.xyxy,f(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,kf.xy,jf);K0=mix(K0,kf.zw,jf);}if((a0&ph)!=0u){X U9=p1(uintBitsToFloat(p0(KB,c0*4u)));c lf=C0(U9,-2.*D0+K0+A0);c mf=C0(U9,-2.*K0+T0+D0);float B1=max(dot(lf,lf),dot(mf,mf));float j4=max(ceil(sqrt(.75*4.*sqrt(B1))),1.);g3=min(uint(j4),g3);}uint da=g3+gf+D2-1u;X A2=Xa(A0,D0,K0,T0);float g1=acos(X8(A2[0],A2[1]));float K4=g1/float(gf);float yc=determinant(X(K0-A0,T0-D0));if(yc==.0) yc=determinant(A2);if(yc<.0) K4=-K4;d7=f(A0,D0);e7=f(K0,T0);m5=f(float(da)-abs(ca-F2.x),float(da),(D2<<10)|g3,K4);n5.xy=UC.xy;if(D2>1u){X zc=X(A2[1],UC.xy);float xj=acos(X8(zc[0],zc[1]));float nf=float(D2);if((a0&(J3|q9))==(K7|q9)){nf-=2.;}float Ac=xj/nf;if(determinant(zc)<.0) Ac=-Ac;n5.z=Ac;}if(ca<ba){a0|=X2;}q8=a0;f I=g9(F2,2./ch,j.Ee);
#ifdef NC
I.y=-I.y;
#endif
Z(d7);Z(e7);Z(m5);Z(n5);Z(q8);z1(I);}
#endif
#ifdef EB
V3 W3 V2(O,LG){q(d7,f);q(e7,f);q(m5,f);q(n5,M);q(q8,uint);c A0=d7.xy;c D0=d7.zw;c K0=e7.xy;c T0=e7.zw;X A2=Xa(A0,D0,K0,T0);float yj=max(floor(m5.x),.0);float da=m5.y;uint of=uint(m5.z);float g3=float(of&0x3ffu);float D2=float(of>>10);float K4=m5.w;uint a0=q8;float y3=da-D2;float w1=yj;if(w1<=y3){a0&=~J3;}else{A0=D0=K0=T0;A2=X(A2[1],n5.xy);g3=1.;w1-=y3;y3=D2;K4=n5.z;bool pf=(a0&q9)!=0u;if(pf||(a0&J3)==p9){y3-=2.;--w1;}bool zj=pf&&(w1==0.||w1==y3);if(zj){a0&=~J3;}else{a0|=K4<.0?y6:wb;}if((a0&J3)>K7){float ea=y3*.5;if(w1<ea) a0|=ub;if(y3>3.&&w1>ea-1.&&w1<ea+1.) a0|=vb;w1=w1<ea?.0:y3;}}c a6;float g1=.0;if(w1==.0||w1==y3){bool H6=w1<y3*.5;a6=H6?A0:T0;g1=gb(H6?A2[0]:A2[1]);}else if((a0&Xd)!=0u){a6=A0;if(w1>=float(n9/2u)) a6=D0;if(w1>=float(n9*3u/4u)) a6=K0;if(w1>=float(n9*7u/8u)) a6=n5.xy;}else{float H1,c6;if(g3==y3){H1=w1/g3;c6=.0;}else{c A,J,z2=D0-A0;c p7=T0-A0;c Y8=K0-D0;J=Y8-z2;A=-3.*Y8+p7;c Aj=J*(g3*2.);c r7=z2*(g3*g3);float fa=.0;float Bj=min(g3-1.,w1);c Bc=normalize(A2[0]);float Cj=-abs(K4);float Dj=(1.+w1)*abs(K4);for(int Cc=rj-1;Cc>=0;--Cc){float v8=fa+exp2(float(Cc));if(v8<=Bj){c Dc=v8*A+Aj;Dc=v8*Dc+r7;float Ej=dot(normalize(Dc),Bc);float Ec=v8*Cj+Dj;Ec=min(Ec,p4);if(Ej>=cos(Ec)) fa=v8;}}float Fj=fa/g3;float qf=w1-fa;float ga=acos(clamp(Bc.x,-1.,1.));ga=Bc.y>=.0?ga:-ga;g1=qf*K4+ga;c P1=c(sin(g1),-cos(g1));float l=dot(P1,A),ha=dot(P1,J),T1=dot(P1,z2);float Gj=max(ha*ha-l*T1,.0);float K2=sqrt(Gj);if(ha>.0) K2=-K2;K2-=ha;float rf=-.5*K2*l;c Fc=(abs(K2*K2+rf)<abs(l*T1+rf))?c(K2,l):c(T1,K2);c6=(Fc.y!=.0)?Fc.x/Fc.y:.0;c6=clamp(c6,.0,1.);if(qf==.0) c6=.0;H1=max(Fj,c6);}c Hj=v6(A0,D0,H1);c sf=v6(D0,K0,H1);c Ij=v6(K0,T0,H1);c tf=v6(Hj,sf,H1);c uf=v6(sf,Ij,H1);a6=v6(tf,uf,H1);if(H1!=c6) g1=gb(uf-tf);}O w8;w8.xy=floatBitsToUint(a6);if((a0&J3)==p9){w8.z=(uint(y3)<<16)|uint(w1);}else{uint Jj=uint(int(round(g1*(65536./h9))))&0xffffu;uint vf=0u;if((a0&J3)>K7){float Kj=clamp(X8(A2[0],A2[1]),-1.,1.);vf=uint(round(sqrt((1.+Kj)*.5)*65535.));}w8.z=(Jj<<16)|vf;}w8.w=a0;L2(w8);}
#endif
