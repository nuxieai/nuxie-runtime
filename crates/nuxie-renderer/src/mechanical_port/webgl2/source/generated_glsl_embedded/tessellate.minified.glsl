#define ai 10
#ifdef CB
h1(g0)K(0,f,LD);K(1,f,MD);K(2,f,UC);
#ifdef ca
K(3,uint,HE);K(4,uint,IE);K(5,uint,JE);K(6,uint,KE);
#else
K(3,Y,TB);
#endif
i1
#endif
q2 I0 W(0,f,D6);I0 W(1,f,E6);I0 W(2,f,S4);I0 W(3,R,T4);V2 W(4,uint,K7);i2
#ifdef CB
Y3 l6(h3,k7,XC);Z3 g4(k7,ha)F4 N4(ed,Kg,OB);N4(fd,Lg,HD);G4 B1(FG,g0,F,B,v){L(v,F,LD,f);L(v,F,MD,f);L(v,F,UC,f);
#ifdef ca
L(v,F,HE,uint);L(v,F,IE,uint);L(v,F,JE,uint);L(v,F,KE,uint);Y TB=Y(HE,IE,JE,KE);
#else
L(v,F,TB,Y);
#endif
U(D6,f);U(E6,f);U(S4,f);U(T4,R);U(K7,uint);c x0=LD.xy;c B0=LD.zw;c F0=MD.xy;c M0=MD.zw;bool me=B<4;float y=me?UC.z:UC.w;int nb=int(me?TB.x:TB.y);
#ifdef Gc
int ne=nb<<16;if(TB.z==0xffffffffu){--ne;}float e9=float(ne>>16);
#else
float e9=float(nb<<16>>16);
#endif
float f9=float(nb>>16);c r2=c((B&1)==0?e9:f9,(B&2)==0?y+1.:y);if((f9-e9)*j.Id<.0){r2.y=2.*y+1.-r2.y;}uint U2=TB.z&0x3ffu;uint oe=(TB.z>>10)&0x3ffu;uint n2=TB.z>>20;uint j0=TB.w;uint I8=j0&ad;uint o0=I8>0u?L0(HD,max(I8,1u)-1u).z:0u;Y P4=o0!=0u?L0(OB,o0*4u+1u):Y(0u,0u,0u,0u);float N2=uintBitsToFloat(P4.z);float O2=uintBitsToFloat(P4.w);if(O2!=.0&&N2==.0){float pe;float bi=If(x0,B0,F0,M0,pe);float ob=O2*(1./va);float ci=Df(x0,B0,F0,M0,pe,ob);float L7=1.-ci*(1./H3);float di=dot(M0-x0,M0-x0)/(ob*ob);float ei=(di-1.)*.5;L7=min(L7,ei);L7=min(L7,.99);float fi=.5*L7;float x=Fc(fi)*-2.+1.;float qe=n8(x*O2,bi);f re=mix(x0.xyxy,M0.xyxy,f(1./3.,1./3.,2./3.,2./3.));B0=mix(B0,re.xy,qe);F0=mix(F0,re.zw,qe);}if((j0&gg)!=0u){e0 W8=L1(uintBitsToFloat(L0(OB,o0*4u)));c se=P0(W8,-2.*B0+F0+x0);c te=P0(W8,-2.*F0+M0+B0);float n1=max(dot(se,se),dot(te,te));float T3=max(ceil(sqrt(.75*4.*sqrt(n1))),1.);U2=min(uint(T3),U2);}uint g9=U2+oe+n2-1u;e0 L2=Z9(x0,B0,F0,M0);float f1=acos(Y9(L2[0],L2[1]));float r4=f1/float(oe);float pb=determinant(e0(F0-x0,M0-B0));if(pb==.0)pb=determinant(L2);if(pb<.0)r4=-r4;D6=f(x0,B0);E6=f(F0,M0);S4=f(float(g9)-abs(f9-r2.x),float(g9),(n2<<10)|U2,r4);T4.xy=UC.xy;if(n2>1u){e0 qb=e0(L2[1],UC.xy);float gi=acos(Y9(qb[0],qb[1]));float ue=float(n2);if((j0&(f4|C8))==(A8|C8)){ue-=2.;}float rb=gi/ue;if(determinant(qb)<.0)rb=-rb;T4.z=rb;}if(f9<e9){j0|=K3;}K7=j0;f X=r8(r2,2./dg,j.Id);
#ifdef RC
X.y=-X.y;
#endif
c0(D6);c0(E6);c0(S4);c0(T4);c0(K7);C1(X);}
#endif
#ifdef EB
I3 J3 f3(H4,GG){r(D6,f);r(E6,f);r(S4,f);r(T4,R);r(K7,uint);c x0=D6.xy;c B0=D6.zw;c F0=E6.xy;c M0=E6.zw;e0 L2=Z9(x0,B0,F0,M0);float hi=max(floor(S4.x),.0);float g9=S4.y;uint ve=uint(S4.z);float U2=float(ve&0x3ffu);float n2=float(ve>>10);float r4=S4.w;uint j0=K7;float U4=g9-n2;float W1=hi;if(W1<=U4){j0&=~f4;}else{x0=B0=F0=M0;L2=e0(L2[1],T4.xy);U2=1.;W1-=U4;U4=n2;r4=T4.z;if((j0&f4)>A8){if(W1<2.5)j0|=wa;if(W1>1.5&&W1<3.5)j0|=Yc;}else if((j0&C8)!=0u||(j0&f4)==B8){U4-=2.;--W1;}j0|=r4<.0?D8:Zc;}c J5;float f1=.0;if(W1==.0||W1==U4||(j0&f4)>A8){bool L8=W1<U4*.5;J5=L8?x0:M0;f1=Ic(L8?L2[0]:L2[1]);}else if((j0&Xc)!=0u){J5=x0;if(W1>=float(ta/2u))J5=B0;if(W1>=float(ta*3u/4u))J5=F0;if(W1>=float(ta*7u/8u))J5=T4.xy;}else{float w1,K5;if(U2==U4){w1=W1/U2;K5=.0;}else{c C,H,l2=B0-x0;c R6=M0-x0;c k8=F0-B0;H=k8-l2;C=-3.*k8+R6;c ii=H*(U2*2.);c T6=l2*(U2*U2);float h9=.0;float ji=min(U2-1.,W1);c sb=normalize(L2[0]);float ki=-abs(r4);float li=(1.+W1)*abs(r4);for(int tb=ai-1;tb>=0;--tb){float M7=h9+exp2(float(tb));if(M7<=ji){c ub=M7*C+ii;ub=M7*ub+T6;float mi=dot(normalize(ub),sb);float vb=M7*ki+li;vb=min(vb,H3);if(mi>=cos(vb))h9=M7;}}float ni=h9/U2;float we=W1-h9;float i9=acos(clamp(sb.x,-1.,1.));i9=sb.y>=.0?i9:-i9;f1=we*r4+i9;c d3=c(sin(f1),-cos(f1));float l=dot(d3,C),j9=dot(d3,H),J1=dot(d3,l2);float oi=max(j9*j9-l*J1,.0);float x2=sqrt(oi);if(j9>.0)x2=-x2;x2-=j9;float xe=-.5*x2*l;c wb=(abs(x2*x2+xe)<abs(l*J1+xe))?c(x2,l):c(J1,x2);K5=(wb.y!=.0)?wb.x/wb.y:.0;K5=clamp(K5,.0,1.);if(we==.0)K5=.0;w1=max(ni,K5);}c pi=f6(x0,B0,w1);c ye=f6(B0,F0,w1);c qi=f6(F0,M0,w1);c ze=f6(pi,ye,w1);c Ae=f6(ye,qi,w1);J5=f6(ze,Ae,w1);if(w1!=K5)f1=Ic(Ae-ze);}H4 N7;N7.xy=fa(J5);if((j0&f4)==B8){N7.z=ga((uint(U4)<<16)|uint(W1));}else{N7.z=fa(mod(f1,v8));}N7.w=ga(j0);M2(N7);}
#endif
