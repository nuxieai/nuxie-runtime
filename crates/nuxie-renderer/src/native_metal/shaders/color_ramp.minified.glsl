#ifdef VERTEX
f1(f0)
#ifdef Sa
K(0,uint,WD);K(1,uint,XD);K(2,uint,YD);K(3,uint,ZD);
#else
K(0,O,JC);
#endif
g1
#endif
w2 F0 X(0,i,p7);l2
#ifdef VERTEX
o4 p4 W4 X4 i Eg(uint n){return ud((O(n,n,n,n)>>O(16,8,0,24))&0xffu)/255.;}x1(OF,f0,B,F,r){
#ifdef Sa
L(r,B,WD,uint);L(r,B,XD,uint);L(r,B,YD,uint);L(r,B,ZD,uint);O JC=O(WD,XD,YD,ZD);
#else
L(r,B,JC,O);
#endif
V(p7,i);int W8=F>>1;float x=float(W8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float Ta=(F&1)==0?.0:1.;if(j.vd<.0){Ta=1.-Ta;}uint q7=JC.y;float y=float(q7&~Fg)+Ta;if((q7&wd)!=0u&&W8==0){if((q7&Ua)!=0u) x=.0;else x-=X8;}if((q7&xd)!=0u&&W8==3){if((q7&Ua)!=0u) x=1.;else x+=X8;}p7=Eg(W8<=1?JC.z:JC.w);e I=Y8(c(x,y),2.,j.vd);
#ifdef POST_INVERT_Y
I.y=-I.y;
#endif
Z(p7);y1(I);}
#endif
#ifdef FRAGMENT
U3 V3 W2(i,PF){q(p7,i);K2(p7);}
#endif
