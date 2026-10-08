#ifdef BB
f1(f0)
#ifdef Qa
K(0,uint,VD);K(1,uint,WD);K(2,uint,XD);K(3,uint,YD);
#else
K(0,O,JC);
#endif
g1
#endif
w2 F0 X(0,i,p7);l2
#ifdef BB
o4 p4 W4 X4 i Bg(uint n){return sd((O(n,n,n,n)>>O(16,8,0,24))&0xffu)/255.;}x1(NF,f0,B,F,r){
#ifdef Qa
L(r,B,VD,uint);L(r,B,WD,uint);L(r,B,XD,uint);L(r,B,YD,uint);O JC=O(VD,WD,XD,YD);
#else
L(r,B,JC,O);
#endif
V(p7,i);int V8=F>>1;float x=float(V8<=1?JC.x&0xffffu:JC.x>>16)/65536.;float Ra=(F&1)==0?.0:1.;if(j.td<.0){Ra=1.-Ra;}uint q7=JC.y;float y=float(q7&~Cg)+Ra;if((q7&ud)!=0u&&V8==0){if((q7&Sa)!=0u) x=.0;else x-=W8;}if((q7&vd)!=0u&&V8==3){if((q7&Sa)!=0u) x=1.;else x+=W8;}p7=Bg(V8<=1?JC.z:JC.w);e I=X8(c(x,y),2.,j.td);
#ifdef MC
I.y=-I.y;
#endif
Z(p7);y1(I);}
#endif
#ifdef EB
U3 V3 W2(i,OF){q(p7,i);K2(p7);}
#endif
