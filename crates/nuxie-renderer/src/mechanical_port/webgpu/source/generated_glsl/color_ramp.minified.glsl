#ifdef VERTEX
e1(e0)
#ifdef bb
K(0,uint,ZD);K(1,uint,AE);K(2,uint,BE);K(3,uint,CE);
#else
K(0,O,KC);
#endif
f1
#endif
m2 H0 W(0,i,y7);M1
#ifdef VERTEX
x4 y4 c5 d5 i Qg(uint k){return Ed((O(k,k,k,k)>>O(16,8,0,24))&0xffu)/255.;}o1(RF,e0,A,C,r){
#ifdef bb
L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);L(r,A,CE,uint);O KC=O(ZD,AE,BE,CE);
#else
L(r,A,KC,O);
#endif
V(y7,i);int h9=C>>1;float x=float(h9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float cb=(C&1)==0?.0:1.;if(j.Fd<.0){cb=1.-cb;}uint z7=KC.y;float y=float(z7&~Rg)+cb;if((z7&Gd)!=0u&&h9==0){if((z7&db)!=0u) x=.0;else x-=f5;}if((z7&Hd)!=0u&&h9==3){if((z7&db)!=0u) x=1.;else x+=f5;}y7=Qg(h9<=1?KC.z:KC.w);e I=i9(c(x,y),2.,j.Fd);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
a0(y7);A1(I);}
#endif
#ifdef FRAGMENT
Y3 Z3 L2(i,SF){q(y7,i);M2(y7);}
#endif
