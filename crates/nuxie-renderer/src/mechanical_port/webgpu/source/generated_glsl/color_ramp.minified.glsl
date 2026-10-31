#ifdef VERTEX
f1(d0)
#ifdef fb
K(0,uint,ZD);K(1,uint,AE);K(2,uint,BE);K(3,uint,CE);
#else
K(0,R,KC);
#endif
g1
#endif
n2 I0 W(0,i,y7);P1
#ifdef VERTEX
A4 B4 h5 i5 i Yg(uint k){return Jd((R(k,k,k,k)>>R(16,8,0,24))&0xffu)/255.;}l1(SF,d0,A,C,r){
#ifdef fb
L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);L(r,A,CE,uint);R KC=R(ZD,AE,BE,CE);
#else
L(r,A,KC,R);
#endif
V(y7,i);int k9=C>>1;float x=float(k9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float gb=(C&1)==0?.0:1.;if(j.Kd<.0){gb=1.-gb;}uint z7=KC.y;float y=float(z7&~Zg)+gb;if((z7&Ld)!=0u&&k9==0){if((z7&hb)!=0u) x=.0;else x-=k5;}if((z7&Md)!=0u&&k9==3){if((z7&hb)!=0u) x=1.;else x+=k5;}y7=Yg(k9<=1?KC.z:KC.w);e I=l9(c(x,y),2.,j.Kd);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
e0(y7);B1(I);}
#endif
#ifdef FRAGMENT
Y3 Z3 r2(i,TF){q(y7,i);v2(y7);}
#endif
