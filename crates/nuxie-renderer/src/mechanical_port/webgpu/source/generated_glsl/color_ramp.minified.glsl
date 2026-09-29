#ifdef VERTEX
f1(f0)
#ifdef Z9
J(0,uint,UD);J(1,uint,VD);J(2,uint,WD);J(3,uint,XD);
#else
J(0,X,JC);
#endif
g1
#endif
p2 H0 V(0,i,V6);h2
#ifdef VERTEX
U3 V3 B4 C4 i Cf(uint j){return pc((X(j,j,j,j)>>X(16,8,0,24))&0xffu)/255.;}y1(LF,f0,F,B,v){
#ifdef Z9
K(v,F,UD,uint);K(v,F,VD,uint);K(v,F,WD,uint);K(v,F,XD,uint);X JC=X(UD,VD,WD,XD);
#else
K(v,F,JC,X);
#endif
T(V6,i);int o8=B>>1;float x=float(o8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float aa=(B&1)==0?.0:1.;if(n.qc<.0){aa=1.-aa;}uint W6=JC.y;float y=float(W6&~Df)+aa;if((W6&rc)!=0u&&o8==0){if((W6&ba)!=0u)x=.0;else x-=sc;}if((W6&tc)!=0u&&o8==3){if((W6&ba)!=0u)x=1.;else x+=sc;}V6=Cf(o8<=1?JC.z:JC.w);f W=p8(c(x,y),2.,n.qc);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
a0(V6);z1(W);}
#endif
#ifdef FRAGMENT
E3 F3 d3(i,MF){r(V6,i);K2(V6);}
#endif
