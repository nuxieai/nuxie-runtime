#define mh 10
#ifdef DB
g1(e0)L(0,g,GD);L(1,g,HD);L(2,g,UC);
#ifdef M3
L(3,uint,CE);L(4,uint,DE);L(5,uint,EE);L(6,uint,FE);
#else
L(3,G,TB);
#endif
h1
#endif
k2 J0 W(0,g,x6);J0 W(1,g,y6);J0 W(2,g,L4);J0 W(3,c0,C5);O2 W(4,uint,F7);f2
#ifdef DB
S3 f6(a3,d7,XC);T3 Z3(d7,aa)z4 G4(Lc,bg,PB);G4(Mc,cg,ED);A4 y1(WF,e0,F,B,r){M(r,F,GD,g);M(r,F,HD,g);M(r,F,UC,g);
#ifdef M3
M(r,F,CE,uint);M(r,F,DE,uint);M(r,F,EE,uint);M(r,F,FE,uint);G TB=G(CE,DE,EE,FE);
#else
M(r,F,TB,G);
#endif
V(x6,g);V(y6,g);V(L4,g);V(C5,c0);V(F7,uint);d y0=GD.xy;d z0=GD.zw;d F0=HD.xy;d I0=HD.zw;bool Md=B<4;float y=Md?UC.z:UC.w;int cb=int(Md?TB.x:TB.y);
#ifdef mc
int Nd=cb<<16;if(TB.z==0xffffffffu){--Nd;}float W8=float(Nd>>16);
#else
float W8=float(cb<<16>>16);
#endif
float X8=float(cb>>16);d m2=d((B&1)==0?W8:X8,(B&2)==0?y+1.:y);if((X8-W8)*n.od<.0){m2.y=2.*y+1.-m2.y;}uint N2=TB.z&0x3ffu;uint Od=(TB.z>>10)&0x3ffu;uint i2=TB.z>>20;uint i0=TB.w;uint C8=i0&Hc;uint o0=C8>0u?N0(ED,max(C8,1u)-1u).z:0u;G I4=o0!=0u?N0(PB,o0*4u+1u):G(0u,0u,0u,0u);float H2=uintBitsToFloat(I4.z);float I2=uintBitsToFloat(I4.w);if(I2!=.0&&H2==.0){float Pd;float nh=af(y0,z0,F0,I0,Pd);float db=I2*(1./ma);float oh=Ve(y0,z0,F0,I0,Pd,db);float G7=1.-oh*(1./B3);float ph=dot(I0-y0,I0-y0)/(db*db);float qh=(ph-1.)*.5;G7=min(G7,qh);G7=min(G7,.99);float rh=.5*G7;float x=lc(rh)*-2.+1.;float Qd=h8(x*I2,nh);g Rd=mix(y0.xyxy,I0.xyxy,g(1./3.,1./3.,2./3.,2./3.));z0=mix(z0,Rd.xy,Qd);F0=mix(F0,Rd.zw,Qd);}if((i0&zf)!=0u){g0 Sd=l2(uintBitsToFloat(N0(PB,o0*4u)));d Td=U0(Sd,-2.*z0+F0+y0);d Ud=U0(Sd,-2.*F0+I0+z0);float l1=max(dot(Td,Td),dot(Ud,Ud));float N3=max(ceil(sqrt(.75*4.*sqrt(l1))),1.);N2=min(uint(N3),N2);}uint Y8=N2+Od+i2-1u;g0 F2=T9(y0,z0,F0,I0);float e1=acos(S9(F2[0],F2[1]));float k4=e1/float(Od);float eb=determinant(g0(F0-y0,I0-z0));if(eb==.0)eb=determinant(F2);if(eb<.0)k4=-k4;x6=g(y0,z0);y6=g(F0,I0);L4=g(float(Y8)-abs(X8-m2.x),float(Y8),(i2<<10)|N2,k4);if(i2>1u){g0 fb=g0(F2[1],UC.xy);float sh=acos(S9(fb[0],fb[1]));float Vd=float(i2);if((i0&(Y3|w8))==(r8|w8)){Vd-=2.;}float gb=sh/Vd;if(determinant(fb)<.0)gb=-gb;C5.xy=UC.xy;C5.z=gb;}if(X8<W8){i0|=E3;}F7=i0;g U=l8(m2,2./wf,n.od);
#ifdef RC
U.y=-U.y;
#endif
a0(x6);a0(y6);a0(L4);a0(C5);a0(F7);z1(U);}
#endif
#ifdef GB
C3 D3 Y2(B4,XF){A(x6,g);A(y6,g);A(L4,g);A(C5,c0);A(F7,uint);d y0=x6.xy;d z0=x6.zw;d F0=y6.xy;d I0=y6.zw;g0 F2=T9(y0,z0,F0,I0);float th=max(floor(L4.x),.0);float Y8=L4.y;uint Wd=uint(L4.z);float N2=float(Wd&0x3ffu);float i2=float(Wd>>10);float k4=L4.w;uint i0=F7;float M4=Y8-i2;float y2=th;if(y2<=M4){i0&=~Y3;}else{y0=z0=F0=I0;F2=g0(F2[1],C5.xy);N2=1.;y2-=M4;M4=i2;k4=C5.z;if((i0&Y3)>r8){if(y2<2.5)i0|=na;if(y2>1.5&&y2<3.5)i0|=Fc;}else if((i0&w8)!=0u||(i0&Y3)==v8){M4-=2.;--y2;}i0|=k4<.0?x8:Gc;}d Z8;float e1=.0;if(y2==.0||y2==M4||(i0&Y3)>r8){bool F8=y2<M4*.5;Z8=F8?y0:I0;e1=oc(F8?F2[0]:F2[1]);}else if((i0&Ec)!=0u){Z8=z0;}else{float r1,D5;if(N2==M4){r1=y2/N2;D5=.0;}else{d C,H,g2=z0-y0;d K6=I0-y0;d e8=F0-z0;H=e8-g2;C=-3.*e8+K6;d uh=H*(N2*2.);d M6=g2*(N2*N2);float a9=.0;float vh=min(N2-1.,y2);d hb=normalize(F2[0]);float wh=-abs(k4);float xh=(1.+y2)*abs(k4);for(int ib=mh-1;ib>=0;--ib){float H7=a9+exp2(float(ib));if(H7<=vh){d jb=H7*C+uh;jb=H7*jb+M6;float yh=dot(normalize(jb),hb);float kb=H7*wh+xh;kb=min(kb,B3);if(yh>=cos(kb))a9=H7;}}float zh=a9/N2;float Xd=y2-a9;float c9=acos(clamp(hb.x,-1.,1.));c9=hb.y>=.0?c9:-c9;e1=Xd*k4+c9;d W2=d(sin(e1),-cos(e1));float m=dot(W2,C),d9=dot(W2,H),G1=dot(W2,g2);float Ah=max(d9*d9-m*G1,.0);float q2=sqrt(Ah);if(d9>.0)q2=-q2;q2-=d9;float Yd=-.5*q2*m;d lb=(abs(q2*q2+Yd)<abs(m*G1+Yd))?d(q2,m):d(G1,q2);D5=(lb.y!=.0)?lb.x/lb.y:.0;D5=clamp(D5,.0,1.);if(Xd==.0)D5=.0;r1=max(zh,D5);}d Bh=Y5(y0,z0,r1);d Zd=Y5(z0,F0,r1);d Ch=Y5(F0,I0,r1);d ae=Y5(Bh,Zd,r1);d be=Y5(Zd,Ch,r1);Z8=Y5(ae,be,r1);if(r1!=D5)e1=oc(be-ae);}B4 I7;I7.xy=Y9(Z8);if((i0&Y3)==v8){I7.z=Z9((uint(M4)<<16)|uint(y2));}else{I7.z=Y9(mod(e1,m8));}I7.w=Z9(i0);G2(I7);}
#endif
