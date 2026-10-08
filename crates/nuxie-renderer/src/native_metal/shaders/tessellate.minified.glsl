#define lj 10
#ifdef VERTEX
f1(f0) K(0,e,MD);K(1,e,ND);K(2,e,TC);
#ifdef Qa
K(3,uint,IE);K(4,uint,JE);K(5,uint,KE);K(6,uint,LE);
#else
K(3,O,WB);
#endif
g1
#endif
w2 F0 X(0,e,U6);F0 X(1,e,V6);F0 X(2,e,i5);F0 X(3,M,j5);g3 X(4,uint,k8);l2
#ifdef VERTEX
o4 F6(q3,F7,ZC);p4 y4(F7,Ta) W4 g5(Xd,ci,KB);g5(Yd,di,BD);X4 x1(HG,f0,B,F,r){L(r,B,MD,e);L(r,B,ND,e);L(r,B,TC,e);
#ifdef Qa
L(r,B,IE,uint);L(r,B,JE,uint);L(r,B,KE,uint);L(r,B,LE,uint);O WB=O(IE,JE,KE,LE);
#else
L(r,B,WB,O);
#endif
V(U6,e);V(V6,e);V(i5,e);V(j5,M);V(k8,uint);c A0=MD.xy;c D0=MD.zw;c J0=ND.xy;c U0=ND.zw;bool Ze=F<4;float y=Ze?TC.z:TC.w;int nc=int(Ze?WB.x:WB.y);
#ifdef Ua
int af=nc<<16;if(WB.z==0xffffffffu){--af;}float T9=float(af>>16);
#else
float T9=float(nc<<16>>16);
#endif
float U9=float(nc>>16);c E2=c((F&1)==0?T9:U9,(F&2)==0?y+1.:y);if((U9-T9)*j.ze<.0){E2.y=2.*y+1.-E2.y;}uint f3=WB.z&0x3ffu;uint bf=(WB.z>>10)&0x3ffu;uint C2=WB.z>>20;uint a0=WB.w;uint B6=a0&mb;uint c0=B6>0u?p0(BD,max(B6,1u)-1u).z:0u;O K3=c0!=0u?p0(KB,c0*4u+1u):O(0u,0u,0u,0u);float B2=uintBitsToFloat(K3.z);float Z2=uintBitsToFloat(K3.w);if(Z2!=.0&&B2==.0){float cf;float mj=yg(A0,D0,J0,U0,cf);float oc=Z2*(1./hb);float nj=tg(A0,D0,J0,U0,cf,oc);float l8=1.-nj*(1./n4);float oj=dot(U0-A0,U0-A0)/(oc*oc);float pj=(oj-1.)*.5;l8=min(l8,pj);l8=min(l8,.99);float qj=.5*l8;float x=yd(qj)*-2.+1.;float df=S8(x*Z2,mj);e ef=mix(A0.xyxy,U0.xyxy,e(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,ef.xy,df);J0=mix(J0,ef.zw,df);}if((a0&jh)!=0u){W y6=p1(uintBitsToFloat(p0(KB,c0*4u)));c ff=y0(y6,-2.*D0+J0+A0);c gf=y0(y6,-2.*J0+U0+D0);float B1=max(dot(ff,ff),dot(gf,gf));float j4=max(ceil(sqrt(.75*4.*sqrt(B1))),1.);f3=min(uint(j4),f3);}uint V9=f3+bf+C2-1u;W z2=Na(A0,D0,J0,U0);float h1=acos(O8(z2[0],z2[1]));float I4=h1/float(bf);float pc=determinant(W(J0-A0,U0-D0));if(pc==.0) pc=determinant(z2);if(pc<.0) I4=-I4;U6=e(A0,D0);V6=e(J0,U0);i5=e(float(V9)-abs(U9-E2.x),float(V9),(C2<<10)|f3,I4);j5.xy=TC.xy;if(C2>1u){W qc=W(z2[1],TC.xy);float rj=acos(O8(qc[0],qc[1]));float hf=float(C2);if((a0&(I3|j9))==(E7|j9)){hf-=2.;}float rc=rj/hf;if(determinant(qc)<.0) rc=-rc;j5.z=rc;}if(U9<T9){a0|=Y2;}k8=a0;e I=X8(E2,2./Wg,j.ze);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
Z(U6);Z(V6);Z(i5);Z(j5);Z(k8);y1(I);}
#endif
#ifdef FRAGMENT
U3 V3 W2(O,IG){q(U6,e);q(V6,e);q(i5,e);q(j5,M);q(k8,uint);c A0=U6.xy;c D0=U6.zw;c J0=V6.xy;c U0=V6.zw;W z2=Na(A0,D0,J0,U0);float sj=max(floor(i5.x),.0);float V9=i5.y;uint jf=uint(i5.z);float f3=float(jf&0x3ffu);float C2=float(jf>>10);float I4=i5.w;uint a0=k8;float x3=V9-C2;float w1=sj;if(w1<=x3){a0&=~I3;}else{A0=D0=J0=U0;z2=W(z2[1],j5.xy);f3=1.;w1-=x3;x3=C2;I4=j5.z;bool kf=(a0&j9)!=0u;if(kf||(a0&I3)==i9){x3-=2.;--w1;}bool tj=kf&&(w1==0.||w1==x3);if(tj){a0&=~I3;}else{a0|=I4<.0?r6:lb;}if((a0&I3)>E7){float W9=x3*.5;if(w1<W9) a0|=jb;if(x3>3.&&w1>W9-1.&&w1<W9+1.) a0|=kb;w1=w1<W9?.0:x3;}}c X5;float h1=.0;if(w1==.0||w1==x3){bool E6=w1<x3*.5;X5=E6?A0:U0;h1=Ad(E6?z2[0]:z2[1]);}else if((a0&Sd)!=0u){X5=A0;if(w1>=float(f9/2u)) X5=D0;if(w1>=float(f9*3u/4u)) X5=J0;if(w1>=float(f9*7u/8u)) X5=j5.xy;}else{float F1,Y5;if(f3==x3){F1=w1/f3;Y5=.0;}else{c A,J,y2=D0-A0;c j7=U0-A0;c P8=J0-D0;J=P8-y2;A=-3.*P8+j7;c uj=J*(f3*2.);c l7=y2*(f3*f3);float X9=.0;float vj=min(f3-1.,w1);c sc=normalize(z2[0]);float wj=-abs(I4);float xj=(1.+w1)*abs(I4);for(int tc=lj-1;tc>=0;--tc){float m8=X9+exp2(float(tc));if(m8<=vj){c uc=m8*A+uj;uc=m8*uc+l7;float yj=dot(normalize(uc),sc);float vc=m8*wj+xj;vc=min(vc,n4);if(yj>=cos(vc)) X9=m8;}}float zj=X9/f3;float lf=w1-X9;float Y9=acos(clamp(sc.x,-1.,1.));Y9=sc.y>=.0?Y9:-Y9;h1=lf*I4+Y9;c P1=c(sin(h1),-cos(h1));float k=dot(P1,A),Z9=dot(P1,J),S1=dot(P1,y2);float Aj=max(Z9*Z9-k*S1,.0);float J2=sqrt(Aj);if(Z9>.0) J2=-J2;J2-=Z9;float mf=-.5*J2*k;c wc=(abs(J2*J2+mf)<abs(k*S1+mf))?c(J2,k):c(S1,J2);Y5=(wc.y!=.0)?wc.x/wc.y:.0;Y5=clamp(Y5,.0,1.);if(lf==.0) Y5=.0;F1=max(zj,Y5);}c Bj=o6(A0,D0,F1);c nf=o6(D0,J0,F1);c Cj=o6(J0,U0,F1);c of=o6(Bj,nf,F1);c pf=o6(nf,Cj,F1);X5=o6(of,pf,F1);if(F1!=Y5) h1=Ad(pf-of);}O n8;n8.xy=floatBitsToUint(X5);if((a0&I3)==i9){n8.z=(uint(x3)<<16)|uint(w1);}else{uint Dj=uint(int(round(h1*(65536./Y8))))&0xffffu;uint qf=0u;if((a0&I3)>E7){float Ej=clamp(O8(z2[0],z2[1]),-1.,1.);qf=uint(round(sqrt((1.+Ej)*.5)*65535.));}n8.z=(Dj<<16)|qf;}n8.w=a0;K2(n8);}
#endif
