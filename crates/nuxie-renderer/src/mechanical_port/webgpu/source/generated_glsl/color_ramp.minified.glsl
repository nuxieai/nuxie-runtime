#ifdef VERTEX
g1(e0)
#ifdef O3
L(0,uint,RD);L(1,uint,SD);L(2,uint,TD);L(3,uint,UD);
#else
L(0,H,LC);
#endif
h1
#endif
m2 H0 X(0,i,W6);g2
#ifdef VERTEX
U3 V3 C4 D4 i mf(uint j){return ic((H(j,j,j,j)>>H(16,8,0,24))&0xffu)/255.;}z1(HF,e0,F,B,v){
#ifdef O3
M(v,F,RD,uint);M(v,F,SD,uint);M(v,F,TD,uint);M(v,F,UD,uint);H LC=H(RD,SD,TD,UD);
#else
M(v,F,LC,H);
#endif
V(W6,i);int o8=B>>1;float x=float(o8<=1?LC.x&0xffffu:LC.x>>16)/65536.;float X9=(B&1)==0?.0:1.;if(m.jc<.0){X9=1.-X9;}uint X6=LC.y;float y=float(X6&~nf)+X9;if((X6&kc)!=0u&&o8==0){if((X6&Y9)!=0u)x=.0;else x-=lc;}if((X6&mc)!=0u&&o8==3){if((X6&Y9)!=0u)x=1.;else x+=lc;}W6=mf(o8<=1?LC.z:LC.w);g W=p8(d(x,y),2.,m.jc);
#ifdef POST_INVERT_Y
W.y=-W.y;
#endif
c0(W6);A1(W);}
#endif
#ifdef FRAGMENT
E3 F3 a3(i,IF){r(W6,i);I2(W6);}
#endif
