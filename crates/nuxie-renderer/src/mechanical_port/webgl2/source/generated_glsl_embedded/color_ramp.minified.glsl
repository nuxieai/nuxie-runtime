#ifdef DB
f1(f0)
#ifdef Z9
J(0,uint,UD);J(1,uint,VD);J(2,uint,WD);J(3,uint,XD);
#else
J(0,X,JC);
#endif
g1
#endif
p2 H0 V(0,i,W6);h2
#ifdef DB
U3 V3 B4 C4 i Cf(uint j){return pc((X(j,j,j,j)>>X(16,8,0,24))&0xffu)/255.;}y1(MF,f0,F,B,v){
#ifdef Z9
K(v,F,UD,uint);K(v,F,VD,uint);K(v,F,WD,uint);K(v,F,XD,uint);X JC=X(UD,VD,WD,XD);
#else
K(v,F,JC,X);
#endif
T(W6,i);int o8=B>>1;float x=float(o8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float aa=(B&1)==0?.0:1.;if(n.qc<.0){aa=1.-aa;}uint X6=JC.y;float y=float(X6&~Df)+aa;if((X6&rc)!=0u&&o8==0){if((X6&ba)!=0u)x=.0;else x-=sc;}if((X6&tc)!=0u&&o8==3){if((X6&ba)!=0u)x=1.;else x+=sc;}W6=Cf(o8<=1?JC.z:JC.w);f W=p8(c(x,y),2.,n.qc);
#ifdef SC
W.y=-W.y;
#endif
a0(W6);z1(W);}
#endif
#ifdef FB
F3 G3 d3(i,NF){r(W6,i);L2(W6);}
#endif
