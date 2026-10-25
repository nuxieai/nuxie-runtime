#define Cj 10
#ifdef BB
e1(e0) K(0,e,QD);K(1,e,RD);K(2,e,UC);
#ifdef bb
K(3,uint,OE);K(4,uint,PE);K(5,uint,QE);K(6,uint,RE);
#else
K(3,O,XB);
#endif
f1
#endif
m2 H0 W(0,e,d7);H0 W(1,e,e7);H0 W(2,e,p5);H0 W(3,M,q5);h3 W(4,uint,v8);M1
#ifdef BB
x4 M6(v3,L7,AD);y4 e4(L7,eb) c5 m5(le,pi,KB);m5(me,qi,DD);d5 o1(LG,e0,A,C,r){L(r,A,QD,e);L(r,A,RD,e);L(r,A,UC,e);
#ifdef bb
L(r,A,OE,uint);L(r,A,PE,uint);L(r,A,QE,uint);L(r,A,RE,uint);O XB=O(OE,PE,QE,RE);
#else
L(r,A,XB,O);
#endif
V(d7,e);V(e7,e);V(p5,e);V(q5,M);V(v8,uint);c A0=QD.xy;c F0=QD.zw;c M0=RD.xy;c U0=RD.zw;bool qf=C<4;float y=qf?UC.z:UC.w;int Ec=int(qf?XB.x:XB.y);
#ifdef fb
int rf=Ec<<16;if(XB.z==0xffffffffu){--rf;}float ca=float(rf>>16);
#else
float ca=float(Ec<<16>>16);
#endif
float da=float(Ec>>16);c F2=c((C&1)==0?ca:da,(C&2)==0?y+1.:y);if((da-ca)*j.Pe<.0){F2.y=2.*y+1.-F2.y;}uint g3=XB.z&0x3ffu;uint sf=(XB.z>>10)&0x3ffu;uint D2=XB.z>>20;uint Z=XB.w;uint H6=Z&yb;uint f0=H6>0u?q0(DD,max(H6,1u)-1u).z:0u;O M3=f0!=0u?q0(KB,f0*4u+1u):O(0u,0u,0u,0u);float C2=uintBitsToFloat(M3.z);float Y2=uintBitsToFloat(M3.w);if(Y2!=.0&&C2==.0){float tf;float Dj=Ng(A0,F0,M0,U0,tf);float Fc=Y2*(1./tb);float Ej=Ig(A0,F0,M0,U0,tf,Fc);float w8=1.-Ej*(1./w4);float Fj=dot(U0-A0,U0-A0)/(Fc*Fc);float Gj=(Fj-1.)*.5;w8=min(w8,Gj);w8=min(w8,.99);float Hj=.5*w8;float x=Kd(Hj)*-2.+1.;float uf=e9(x*Y2,Dj);e vf=mix(A0.xyxy,U0.xyxy,e(1./3.,1./3.,2./3.,2./3.));F0=mix(F0,vf.xy,uf);M0=mix(M0,vf.zw,uf);}if((Z&yh)!=0u){X V9=q1(uintBitsToFloat(q0(KB,f0*4u)));c wf=D0(V9,-2.*F0+M0+A0);c xf=D0(V9,-2.*M0+U0+F0);float C1=max(dot(wf,wf),dot(xf,xf));float l4=max(ceil(sqrt(.75*4.*sqrt(C1))),1.);g3=min(uint(l4),g3);}uint ea=g3+sf+D2-1u;X A2=Ya(A0,F0,M0,U0);float g1=acos(Z8(A2[0],A2[1]));float O4=g1/float(sf);float Gc=determinant(X(M0-A0,U0-F0));if(Gc==.0) Gc=determinant(A2);if(Gc<.0) O4=-O4;d7=e(A0,F0);e7=e(M0,U0);p5=e(float(ea)-abs(da-F2.x),float(ea),(D2<<10)|g3,O4);q5.xy=UC.xy;if(D2>1u){X Hc=X(A2[1],UC.xy);float Ij=acos(Z8(Hc[0],Hc[1]));float yf=float(D2);if((Z&(r3|r9))==(K7|r9)){yf-=2.;}float Ic=Ij/yf;if(determinant(Hc)<.0) Ic=-Ic;q5.z=Ic;}if(da<ca){Z|=X2;}v8=Z;e I=i9(F2,2./lh,j.Pe);
#ifdef NC
I.y=-I.y;
#endif
a0(d7);a0(e7);a0(p5);a0(q5);a0(v8);A1(I);}
#endif
#ifdef DB
Y3 Z3 L2(O,MG){q(d7,e);q(e7,e);q(p5,e);q(q5,M);q(v8,uint);c A0=d7.xy;c F0=d7.zw;c M0=e7.xy;c U0=e7.zw;X A2=Ya(A0,F0,M0,U0);float Jj=max(floor(p5.x),.0);float ea=p5.y;uint zf=uint(p5.z);float g3=float(zf&0x3ffu);float D2=float(zf>>10);float O4=p5.w;uint Z=v8;float z3=ea-D2;float y1=Jj;if(y1<=z3){Z&=~r3;}else{A0=F0=M0=U0;A2=X(A2[1],q5.xy);g3=1.;y1-=z3;z3=D2;O4=q5.z;bool Af=(Z&r9)!=0u;if(Af||(Z&r3)==q9){z3-=2.;--y1;}bool Kj=Af&&(y1==0.||y1==z3);if(Kj){Z&=~r3;}else if((Z&r3)!=Bh){Z|=O4<.0?C6:xb;}if((Z&r3)>K7){float fa=z3*.5;if(y1<fa) Z|=vb;if(z3>3.&&y1>fa-1.&&y1<fa+1.) Z|=wb;y1=y1<fa?.0:z3;}}c e6;float g1=.0;if(y1==.0||y1==z3){bool L6=y1<z3*.5;e6=L6?A0:U0;g1=hb(L6?A2[0]:A2[1]);}else if((Z&ee)!=0u){e6=A0;if(y1>=float(o9/2u)) e6=F0;if(y1>=float(o9*3u/4u)) e6=M0;if(y1>=float(o9*7u/8u)) e6=q5.xy;}else{float J1,f6;if(g3==z3){J1=y1/g3;f6=.0;}else{c B,J,z2=F0-A0;c p7=U0-A0;c a9=M0-F0;J=a9-z2;B=-3.*a9+p7;c Lj=J*(g3*2.);c r7=z2*(g3*g3);float ga=.0;float Mj=min(g3-1.,y1);c Jc=normalize(A2[0]);float Nj=-abs(O4);float Oj=(1.+y1)*abs(O4);for(int Kc=Cj-1;Kc>=0;--Kc){float x8=ga+exp2(float(Kc));if(x8<=Mj){c Lc=x8*B+Lj;Lc=x8*Lc+r7;float Pj=dot(normalize(Lc),Jc);float Mc=x8*Nj+Oj;Mc=min(Mc,w4);if(Pj>=cos(Mc)) ga=x8;}}float Qj=ga/g3;float Bf=y1-ga;float ha=acos(clamp(Jc.x,-1.,1.));ha=Jc.y>=.0?ha:-ha;g1=Bf*O4+ha;c S1=c(sin(g1),-cos(g1));float l=dot(S1,B),ia=dot(S1,J),W1=dot(S1,z2);float Rj=max(ia*ia-l*W1,.0);float K2=sqrt(Rj);if(ia>.0) K2=-K2;K2-=ia;float Cf=-.5*K2*l;c Nc=(abs(K2*K2+Cf)<abs(l*W1+Cf))?c(K2,l):c(W1,K2);f6=(Nc.y!=.0)?Nc.x/Nc.y:.0;f6=clamp(f6,.0,1.);if(Bf==.0) f6=.0;J1=max(Qj,f6);}c Sj=y6(A0,F0,J1);c Df=y6(F0,M0,J1);c Tj=y6(M0,U0,J1);c Ef=y6(Sj,Df,J1);c Ff=y6(Df,Tj,J1);e6=y6(Ef,Ff,J1);if(J1!=f6) g1=hb(Ff-Ef);}O y8;y8.xy=floatBitsToUint(e6);if((Z&r3)==q9){y8.z=(uint(z3)<<16)|uint(y1);}else{uint Uj=uint(int(round(g1*(65536./A7))))&0xffffu;uint Gf=0u;if((Z&r3)>K7){float Vj=clamp(Z8(A2[0],A2[1]),-1.,1.);Gf=uint(round(sqrt((1.+Vj)*.5)*65535.));}y8.z=(Uj<<16)|Gf;}y8.w=Z;M2(y8);}
#endif
