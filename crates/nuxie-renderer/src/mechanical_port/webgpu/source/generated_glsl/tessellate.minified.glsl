#define pj 10
#ifdef VERTEX
d1(f0) K(0,f,ND);K(1,f,OD);K(2,f,SC);
#ifdef Xa
K(3,uint,JE);K(4,uint,KE);K(5,uint,LE);K(6,uint,ME);
#else
K(3,O,WB);
#endif
e1
#endif
v2 F0 W(0,f,Z6);F0 W(1,f,a7);F0 W(2,f,m5);F0 W(3,M,n5);h3 W(4,uint,o8);k2
#ifdef VERTEX
q4 I6(r3,J7,YC);r4 a4(J7,ab) Y4 j5(ae,ei,KB);j5(be,fi,BD);Z4 w1(IG,f0,B,F,r){L(r,B,ND,f);L(r,B,OD,f);L(r,B,SC,f);
#ifdef Xa
L(r,B,JE,uint);L(r,B,KE,uint);L(r,B,LE,uint);L(r,B,ME,uint);O WB=O(JE,KE,LE,ME);
#else
L(r,B,WB,O);
#endif
V(Z6,f);V(a7,f);V(m5,f);V(n5,M);V(o8,uint);c z0=ND.xy;c D0=ND.zw;c K0=OD.xy;c T0=OD.zw;bool cf=F<4;float y=cf?SC.z:SC.w;int wc=int(cf?WB.x:WB.y);
#ifdef bb
int df=wc<<16;if(WB.z==0xffffffffu){--df;}float Z9=float(df>>16);
#else
float Z9=float(wc<<16>>16);
#endif
float aa=float(wc>>16);c E2=c((F&1)==0?Z9:aa,(F&2)==0?y+1.:y);if((aa-Z9)*j.Ce<.0){E2.y=2.*y+1.-E2.y;}uint g3=WB.z&0x3ffu;uint ef=(WB.z>>10)&0x3ffu;uint C2=WB.z>>20;uint a0=WB.w;uint D6=a0&ub;uint c0=D6>0u?p0(BD,max(D6,1u)-1u).z:0u;O L3=c0!=0u?p0(KB,c0*4u+1u):O(0u,0u,0u,0u);float B2=uintBitsToFloat(L3.z);float Y2=uintBitsToFloat(L3.w);if(Y2!=.0&&B2==.0){float ff;float qj=Cg(z0,D0,K0,T0,ff);float xc=Y2*(1./pb);float rj=xg(z0,D0,K0,T0,ff,xc);float p8=1.-rj*(1./p4);float sj=dot(T0-z0,T0-z0)/(xc*xc);float tj=(sj-1.)*.5;p8=min(p8,tj);p8=min(p8,.99);float uj=.5*p8;float x=Cd(uj)*-2.+1.;float gf=Z8(x*Y2,qj);f hf=mix(z0.xyxy,T0.xyxy,f(1./3.,1./3.,2./3.,2./3.));D0=mix(D0,hf.xy,gf);K0=mix(K0,hf.zw,gf);}if((a0&nh)!=0u){X S9=o1(uintBitsToFloat(p0(KB,c0*4u)));c jf=B0(S9,-2.*D0+K0+z0);c kf=B0(S9,-2.*K0+T0+D0);float A1=max(dot(jf,jf),dot(kf,kf));float j4=max(ceil(sqrt(.75*4.*sqrt(A1))),1.);g3=min(uint(j4),g3);}uint ba=g3+ef+C2-1u;X z2=Ua(z0,D0,K0,T0);float f1=acos(V8(z2[0],z2[1]));float K4=f1/float(ef);float yc=determinant(X(K0-z0,T0-D0));if(yc==.0) yc=determinant(z2);if(yc<.0) K4=-K4;Z6=f(z0,D0);a7=f(K0,T0);m5=f(float(ba)-abs(aa-E2.x),float(ba),(C2<<10)|g3,K4);n5.xy=SC.xy;if(C2>1u){X zc=X(z2[1],SC.xy);float vj=acos(V8(zc[0],zc[1]));float lf=float(C2);if((a0&(J3|o9))==(I7|o9)){lf-=2.;}float Ac=vj/lf;if(determinant(zc)<.0) Ac=-Ac;n5.z=Ac;}if(aa<Z9){a0|=X2;}o8=a0;f I=e9(E2,2./ah,j.Ce);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
Z(Z6);Z(a7);Z(m5);Z(n5);Z(o8);x1(I);}
#endif
#ifdef FRAGMENT
V3 W3 V2(O,JG){q(Z6,f);q(a7,f);q(m5,f);q(n5,M);q(o8,uint);c z0=Z6.xy;c D0=Z6.zw;c K0=a7.xy;c T0=a7.zw;X z2=Ua(z0,D0,K0,T0);float wj=max(floor(m5.x),.0);float ba=m5.y;uint mf=uint(m5.z);float g3=float(mf&0x3ffu);float C2=float(mf>>10);float K4=m5.w;uint a0=o8;float y3=ba-C2;float v1=wj;if(v1<=y3){a0&=~J3;}else{z0=D0=K0=T0;z2=X(z2[1],n5.xy);g3=1.;v1-=y3;y3=C2;K4=n5.z;bool nf=(a0&o9)!=0u;if(nf||(a0&J3)==n9){y3-=2.;--v1;}bool xj=nf&&(v1==0.||v1==y3);if(xj){a0&=~J3;}else{a0|=K4<.0?y6:tb;}if((a0&J3)>I7){float ca=y3*.5;if(v1<ca) a0|=rb;if(y3>3.&&v1>ca-1.&&v1<ca+1.) a0|=sb;v1=v1<ca?.0:y3;}}c a6;float f1=.0;if(v1==.0||v1==y3){bool H6=v1<y3*.5;a6=H6?z0:T0;f1=db(H6?z2[0]:z2[1]);}else if((a0&Vd)!=0u){a6=z0;if(v1>=float(l9/2u)) a6=D0;if(v1>=float(l9*3u/4u)) a6=K0;if(v1>=float(l9*7u/8u)) a6=n5.xy;}else{float E1,c6;if(g3==y3){E1=v1/g3;c6=.0;}else{c A,J,y2=D0-z0;c n7=T0-z0;c W8=K0-D0;J=W8-y2;A=-3.*W8+n7;c yj=J*(g3*2.);c p7=y2*(g3*g3);float da=.0;float zj=min(g3-1.,v1);c Bc=normalize(z2[0]);float Aj=-abs(K4);float Bj=(1.+v1)*abs(K4);for(int Cc=pj-1;Cc>=0;--Cc){float q8=da+exp2(float(Cc));if(q8<=zj){c Dc=q8*A+yj;Dc=q8*Dc+p7;float Cj=dot(normalize(Dc),Bc);float Ec=q8*Aj+Bj;Ec=min(Ec,p4);if(Cj>=cos(Ec)) da=q8;}}float Dj=da/g3;float of=v1-da;float ea=acos(clamp(Bc.x,-1.,1.));ea=Bc.y>=.0?ea:-ea;f1=of*K4+ea;c N1=c(sin(f1),-cos(f1));float m=dot(N1,A),fa=dot(N1,J),R1=dot(N1,y2);float Ej=max(fa*fa-m*R1,.0);float J2=sqrt(Ej);if(fa>.0) J2=-J2;J2-=fa;float pf=-.5*J2*m;c Fc=(abs(J2*J2+pf)<abs(m*R1+pf))?c(J2,m):c(R1,J2);c6=(Fc.y!=.0)?Fc.x/Fc.y:.0;c6=clamp(c6,.0,1.);if(of==.0) c6=.0;E1=max(Dj,c6);}c Fj=v6(z0,D0,E1);c qf=v6(D0,K0,E1);c Gj=v6(K0,T0,E1);c rf=v6(Fj,qf,E1);c sf=v6(qf,Gj,E1);a6=v6(rf,sf,E1);if(E1!=c6) f1=db(sf-rf);}O r8;r8.xy=floatBitsToUint(a6);if((a0&J3)==n9){r8.z=(uint(y3)<<16)|uint(v1);}else{uint Hj=uint(int(round(f1*(65536./f9))))&0xffffu;uint tf=0u;if((a0&J3)>I7){float Ij=clamp(V8(z2[0],z2[1]),-1.,1.);tf=uint(round(sqrt((1.+Ij)*.5)*65535.));}r8.z=(Hj<<16)|tf;}r8.w=a0;K2(r8);}
#endif
