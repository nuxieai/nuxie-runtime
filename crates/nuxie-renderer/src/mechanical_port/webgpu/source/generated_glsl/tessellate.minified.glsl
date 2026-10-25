#define wj 10
#ifdef VERTEX
e1(e0) K(0,e,QD);K(1,e,RD);K(2,e,UC);
#ifdef ab
K(3,uint,NE);K(4,uint,OE);K(5,uint,PE);K(6,uint,QE);
#else
K(3,O,XB);
#endif
f1
#endif
m2 F0 W(0,e,e7);F0 W(1,e,f7);F0 W(2,e,n5);F0 W(3,M,o5);h3 W(4,uint,q8);M1
#ifdef VERTEX
v4 M6(v3,L7,AD);w4 d4(L7,db) a5 k5(fe,ki,KB);k5(ge,li,DD);c5 o1(KG,e0,A,C,r){L(r,A,QD,e);L(r,A,RD,e);L(r,A,UC,e);
#ifdef ab
L(r,A,NE,uint);L(r,A,OE,uint);L(r,A,PE,uint);L(r,A,QE,uint);O XB=O(NE,OE,PE,QE);
#else
L(r,A,XB,O);
#endif
V(e7,e);V(f7,e);V(n5,e);V(o5,M);V(q8,uint);c A0=QD.xy;c D0=QD.zw;c L0=RD.xy;c T0=RD.zw;bool hf=C<4;float y=hf?UC.z:UC.w;int Ac=int(hf?XB.x:XB.y);
#ifdef eb
int jf=Ac<<16;if(XB.z==0xffffffffu){--jf;}float ba=float(jf>>16);
#else
float ba=float(Ac<<16>>16);
#endif
float ca=float(Ac>>16);c F2=c((C&1)==0?ba:ca,(C&2)==0?y+1.:y);if((ca-ba)*j.He<.0){F2.y=2.*y+1.-F2.y;}uint g3=XB.z&0x3ffu;uint kf=(XB.z>>10)&0x3ffu;uint D2=XB.z>>20;uint Z=XB.w;uint H6=Z&xb;uint f0=H6>0u?q0(DD,max(H6,1u)-1u).z:0u;O L3=f0!=0u?q0(KB,f0*4u+1u):O(0u,0u,0u,0u);float C2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);if(Y2!=.0&&C2==.0){float lf;float xj=Hg(A0,D0,L0,T0,lf);float Bc=Y2*(1./sb);float yj=Cg(A0,D0,L0,T0,lf,Bc);float r8=1.-yj*(1./r4);float zj=dot(T0-A0,T0-A0)/(Bc*Bc);float Aj=(zj-1.)*.5;r8=min(r8,Aj);r8=min(r8,.99);float Bj=.5*r8;float x=Hd(Bj)*-2.+1.;float mf=c9(x*Y2,xj);e nf=mix(A0.xyxy,T0.xyxy,e(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,nf.xy,mf);L0=mix(L0,nf.zw,mf);}if((Z&sh)!=0u){X U9=q1(uintBitsToFloat(q0(KB,f0*4u)));c of=C0(U9,-2.*D0+L0+A0);c pf=C0(U9,-2.*L0+T0+D0);float C1=max(dot(of,of),dot(pf,pf));float l4=max(ceil(sqrt(.75*4.*sqrt(C1))),1.);g3=min(uint(l4),g3);}uint da=g3+kf+D2-1u;X A2=Xa(A0,D0,L0,T0);float g1=acos(X8(A2[0],A2[1]));float M4=g1/float(kf);float Cc=determinant(X(L0-A0,T0-D0));if(Cc==.0) Cc=determinant(A2);if(Cc<.0) M4=-M4;e7=e(A0,D0);f7=e(L0,T0);n5=e(float(da)-abs(ca-F2.x),float(da),(D2<<10)|g3,M4);o5.xy=UC.xy;if(D2>1u){X Dc=X(A2[1],UC.xy);float Cj=acos(X8(Dc[0],Dc[1]));float qf=float(D2);if((Z&(r3|q9))==(K7|q9)){qf-=2.;}float Ec=Cj/qf;if(determinant(Dc)<.0) Ec=-Ec;o5.z=Ec;}if(ca<ba){Z|=X2;}q8=Z;e I=g9(F2,2./fh,j.He);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
a0(e7);a0(f7);a0(n5);a0(o5);a0(q8);A1(I);}
#endif
#ifdef FRAGMENT
X3 Y3 L2(O,LG){q(e7,e);q(f7,e);q(n5,e);q(o5,M);q(q8,uint);c A0=e7.xy;c D0=e7.zw;c L0=f7.xy;c T0=f7.zw;X A2=Xa(A0,D0,L0,T0);float Dj=max(floor(n5.x),.0);float da=n5.y;uint rf=uint(n5.z);float g3=float(rf&0x3ffu);float D2=float(rf>>10);float M4=n5.w;uint Z=q8;float z3=da-D2;float y1=Dj;if(y1<=z3){Z&=~r3;}else{A0=D0=L0=T0;A2=X(A2[1],o5.xy);g3=1.;y1-=z3;z3=D2;M4=o5.z;bool sf=(Z&q9)!=0u;if(sf||(Z&r3)==p9){z3-=2.;--y1;}bool Ej=sf&&(y1==0.||y1==z3);if(Ej){Z&=~r3;}else if((Z&r3)!=vh){Z|=M4<.0?C6:wb;}if((Z&r3)>K7){float ea=z3*.5;if(y1<ea) Z|=ub;if(z3>3.&&y1>ea-1.&&y1<ea+1.) Z|=vb;y1=y1<ea?.0:z3;}}c e6;float g1=.0;if(y1==.0||y1==z3){bool L6=y1<z3*.5;e6=L6?A0:T0;g1=gb(L6?A2[0]:A2[1]);}else if((Z&be)!=0u){e6=A0;if(y1>=float(n9/2u)) e6=D0;if(y1>=float(n9*3u/4u)) e6=L0;if(y1>=float(n9*7u/8u)) e6=o5.xy;}else{float J1,f6;if(g3==z3){J1=y1/g3;f6=.0;}else{c B,J,z2=D0-A0;c q7=T0-A0;c Y8=L0-D0;J=Y8-z2;B=-3.*Y8+q7;c Fj=J*(g3*2.);c v7=z2*(g3*g3);float fa=.0;float Gj=min(g3-1.,y1);c Fc=normalize(A2[0]);float Hj=-abs(M4);float Ij=(1.+y1)*abs(M4);for(int Gc=wj-1;Gc>=0;--Gc){float v8=fa+exp2(float(Gc));if(v8<=Gj){c Hc=v8*B+Fj;Hc=v8*Hc+v7;float Jj=dot(normalize(Hc),Fc);float Ic=v8*Hj+Ij;Ic=min(Ic,r4);if(Jj>=cos(Ic)) fa=v8;}}float Kj=fa/g3;float tf=y1-fa;float ga=acos(clamp(Fc.x,-1.,1.));ga=Fc.y>=.0?ga:-ga;g1=tf*M4+ga;c S1=c(sin(g1),-cos(g1));float l=dot(S1,B),ha=dot(S1,J),W1=dot(S1,z2);float Lj=max(ha*ha-l*W1,.0);float K2=sqrt(Lj);if(ha>.0) K2=-K2;K2-=ha;float uf=-.5*K2*l;c Jc=(abs(K2*K2+uf)<abs(l*W1+uf))?c(K2,l):c(W1,K2);f6=(Jc.y!=.0)?Jc.x/Jc.y:.0;f6=clamp(f6,.0,1.);if(tf==.0) f6=.0;J1=max(Kj,f6);}c Mj=y6(A0,D0,J1);c vf=y6(D0,L0,J1);c Nj=y6(L0,T0,J1);c wf=y6(Mj,vf,J1);c xf=y6(vf,Nj,J1);e6=y6(wf,xf,J1);if(J1!=f6) g1=gb(xf-wf);}O w8;w8.xy=floatBitsToUint(e6);if((Z&r3)==p9){w8.z=(uint(z3)<<16)|uint(y1);}else{uint Oj=uint(int(round(g1*(65536./h9))))&0xffffu;uint yf=0u;if((Z&r3)>K7){float Pj=clamp(X8(A2[0],A2[1]),-1.,1.);yf=uint(round(sqrt((1.+Pj)*.5)*65535.));}w8.z=(Oj<<16)|yf;}w8.w=Z;M2(w8);}
#endif
