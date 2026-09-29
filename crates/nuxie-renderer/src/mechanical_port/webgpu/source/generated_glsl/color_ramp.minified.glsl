#ifdef VERTEX
g1(e0)
#ifdef U9
O(0,uint,QD);O(1,uint,RD);O(2,uint,SD);O(3,uint,TD);
#else
O(0,X,JC);
#endif
h1
#endif
m2 H0 W(0,i,V6);g2
#ifdef VERTEX
T3 U3 B4 C4 i mf(uint j){return ic((X(j,j,j,j)>>X(16,8,0,24))&0xffu)/255.;}z1(GF,e0,F,B,A){
#ifdef U9
P(A,F,QD,uint);P(A,F,RD,uint);P(A,F,SD,uint);P(A,F,TD,uint);X JC=X(QD,RD,SD,TD);
#else
P(A,F,JC,X);
#endif
U(V6,i);int n8=B>>1;float x=float(n8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float V9=(B&1)==0?.0:1.;if(m.jc<.0){V9=1.-V9;}uint W6=JC.y;float y=float(W6&~nf)+V9;if((W6&kc)!=0u&&n8==0){if((W6&W9)!=0u)x=.0;else x-=lc;}if((W6&mc)!=0u&&n8==3){if((W6&W9)!=0u)x=1.;else x+=lc;}V6=mf(n8<=1?JC.z:JC.w);g V=o8(d(x,y),2.,m.jc);
#ifdef POST_INVERT_Y
V.y=-V.y;
#endif
c0(V6);A1(V);}
#endif
#ifdef FRAGMENT
D3 E3 a3(i,HF){r(V6,i);I2(V6);}
#endif
