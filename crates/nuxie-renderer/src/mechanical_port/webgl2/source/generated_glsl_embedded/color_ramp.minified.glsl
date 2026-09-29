#ifdef DB
g1(e0)
#ifdef O3
L(0,uint,RD);L(1,uint,SD);L(2,uint,TD);L(3,uint,UD);
#else
L(0,G,LC);
#endif
h1
#endif
m2 H0 X(0,i,V6);g2
#ifdef DB
U3 V3 C4 D4 i mf(uint j){return hc((G(j,j,j,j)>>G(16,8,0,24))&0xffu)/255.;}z1(HF,e0,F,B,v){
#ifdef O3
M(v,F,RD,uint);M(v,F,SD,uint);M(v,F,TD,uint);M(v,F,UD,uint);G LC=G(RD,SD,TD,UD);
#else
M(v,F,LC,G);
#endif
V(V6,i);int n8=B>>1;float x=float(n8<=1?LC.x&0xffffu:LC.x>>16)/65536.;float W9=(B&1)==0?.0:1.;if(m.ic<.0){W9=1.-W9;}uint W6=LC.y;float y=float(W6&~nf)+W9;if((W6&jc)!=0u&&n8==0){if((W6&X9)!=0u)x=.0;else x-=kc;}if((W6&lc)!=0u&&n8==3){if((W6&X9)!=0u)x=1.;else x+=kc;}V6=mf(n8<=1?LC.z:LC.w);g W=o8(d(x,y),2.,m.ic);
#ifdef SC
W.y=-W.y;
#endif
c0(V6);A1(W);}
#endif
#ifdef GB
E3 F3 a3(i,IF){r(V6,i);I2(V6);}
#endif
