#ifdef BB
d1(e0)
#ifdef cb
K(0,uint,YD);K(1,uint,ZD);K(2,uint,AE);K(3,uint,BE);
#else
K(0,R,KC);
#endif
e1
#endif
n2 H0 W(0,i,y7);N1
#ifdef BB
x4 y4 c5 d5 i Qg(uint k){return Gd((R(k,k,k,k)>>R(16,8,0,24))&0xffu)/255.;}n1(QF,e0,A,C,r){
#ifdef cb
L(r,A,YD,uint);L(r,A,ZD,uint);L(r,A,AE,uint);L(r,A,BE,uint);R KC=R(YD,ZD,AE,BE);
#else
L(r,A,KC,R);
#endif
V(y7,i);int g9=C>>1;float x=float(g9<=1?KC.x&0xffffu:KC.x>>16)/65536.;float db=(C&1)==0?.0:1.;if(j.Hd<.0){db=1.-db;}uint z7=KC.y;float y=float(z7&~Rg)+db;if((z7&Id)!=0u&&g9==0){if((z7&eb)!=0u) x=.0;else x-=f5;}if((z7&Jd)!=0u&&g9==3){if((z7&eb)!=0u) x=1.;else x+=f5;}y7=Qg(g9<=1?KC.z:KC.w);e I=h9(c(x,y),2.,j.Hd);
#ifdef NC
I.y=-I.y;
#endif
a0(y7);A1(I);}
#endif
#ifdef DB
Y3 Z3 M2(i,RF){q(y7,i);N2(y7);}
#endif
