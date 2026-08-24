#ifdef DB
g1(e0)
#ifdef M3
L(0,uint,OD);L(1,uint,PD);L(2,uint,QD);L(3,uint,RD);
#else
L(0,G,KC);
#endif
h1
#endif
k2 J0 W(0,i,R6);f2
#ifdef DB
S3 T3 z4 A4 i df(uint j){return dc((G(j,j,j,j)>>G(16,8,0,24))&0xffu)/255.;}y1(EF,e0,F,B,r){
#ifdef M3
M(r,F,OD,uint);M(r,F,PD,uint);M(r,F,QD,uint);M(r,F,RD,uint);G KC=G(OD,PD,QD,RD);
#else
M(r,F,KC,G);
#endif
V(R6,i);int k8=B>>1;float x=float(k8<=1?KC.x&0xffffu:KC.x>>16)/65536.;float W9=(B&1)==0?.0:1.;if(n.ec<.0){W9=1.-W9;}uint S6=KC.y;float y=float(S6&~ef)+W9;if((S6&fc)!=0u&&k8==0){if((S6&X9)!=0u)x=.0;else x-=gc;}if((S6&hc)!=0u&&k8==3){if((S6&X9)!=0u)x=1.;else x+=gc;}R6=df(k8<=1?KC.z:KC.w);g U=l8(d(x,y),2.,n.ec);
#ifdef RC
U.y=-U.y;
#endif
a0(R6);z1(U);}
#endif
#ifdef GB
C3 D3 Y2(i,FF){A(R6,i);G2(R6);}
#endif
