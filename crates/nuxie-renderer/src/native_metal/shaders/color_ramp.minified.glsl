#ifdef VERTEX
g1(e0)
#ifdef O3
L(0,uint,QD);L(1,uint,RD);L(2,uint,SD);L(3,uint,TD);
#else
L(0,G,LC);
#endif
h1
#endif
l2 H0 X(0,i,T6);f2
#ifdef VERTEX
U3 V3 B4 C4 i gf(uint j){return fc((G(j,j,j,j)>>G(16,8,0,24))&0xffu)/255.;}y1(GF,e0,F,B,v){
#ifdef O3
M(v,F,QD,uint);M(v,F,RD,uint);M(v,F,SD,uint);M(v,F,TD,uint);G LC=G(QD,RD,SD,TD);
#else
M(v,F,LC,G);
#endif
V(T6,i);int m8=B>>1;float x=float(m8<=1?LC.x&0xffffu:LC.x>>16)/65536.;float W9=(B&1)==0?.0:1.;if(m.gc<.0){W9=1.-W9;}uint U6=LC.y;float y=float(U6&~hf)+W9;if((U6&hc)!=0u&&m8==0){if((U6&X9)!=0u)x=.0;else x-=ic;}if((U6&jc)!=0u&&m8==3){if((U6&X9)!=0u)x=1.;else x+=ic;}T6=gf(m8<=1?LC.z:LC.w);g W=n8(d(x,y),2.,m.gc);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
a0(T6);z1(W);}
#endif
#ifdef FRAGMENT
E3 F3 a3(i,HF){r(T6,i);I2(T6);}
#endif
