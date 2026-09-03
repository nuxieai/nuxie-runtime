#ifdef DB
g1(e0)
#ifdef O3
L(0,uint,QD);L(1,uint,RD);L(2,uint,SD);L(3,uint,TD);
#else
L(0,G,LC);
#endif
h1
#endif
m2 H0 X(0,i,U6);g2
#ifdef DB
U3 V3 B4 C4 i hf(uint j){return gc((G(j,j,j,j)>>G(16,8,0,24))&0xffu)/255.;}y1(GF,e0,F,B,v){
#ifdef O3
M(v,F,QD,uint);M(v,F,RD,uint);M(v,F,SD,uint);M(v,F,TD,uint);G LC=G(QD,RD,SD,TD);
#else
M(v,F,LC,G);
#endif
V(U6,i);int n8=B>>1;float x=float(n8<=1?LC.x&0xffffu:LC.x>>16)/65536.;float W9=(B&1)==0?.0:1.;if(m.hc<.0){W9=1.-W9;}uint V6=LC.y;float y=float(V6&~jf)+W9;if((V6&ic)!=0u&&n8==0){if((V6&X9)!=0u)x=.0;else x-=jc;}if((V6&kc)!=0u&&n8==3){if((V6&X9)!=0u)x=1.;else x+=jc;}U6=hf(n8<=1?LC.z:LC.w);g W=o8(d(x,y),2.,m.hc);
#ifdef SC
W.y=-W.y;
#endif
a0(U6);z1(W);}
#endif
#ifdef GB
E3 F3 a3(i,HF){r(U6,i);I2(U6);}
#endif
