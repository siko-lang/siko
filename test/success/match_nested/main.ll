@.str_0 = private unnamed_addr constant [1 x i8] c"4", align 1
@.str_1 = private unnamed_addr constant [3 x i8] c"foo", align 1
%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.Main_Foo = type { i32, [4 x i8] }

%struct.Main_Foo_Bar = type { i32, %struct.siko_Tuple_Bool_Bool }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool = type { %struct.Bool_Bool }

%struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo = type { %struct.Bool_Bool, %struct.Int_Int, %struct.String_String, %struct.Main_Foo }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo(ptr noundef %f0, ptr noundef %f1, ptr noundef %f2, ptr noundef %f3, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, align 8
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field1, ptr align 8 %f1, i64 8, i1 false)
   %field2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 2
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field2, ptr align 8 %f2, i64 16, i1 false)
   %field3 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 3
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field3, ptr align 4 %f3, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b38i2 = alloca %struct.Int_Int, align 8
   %v_5 = alloca %struct.Int_Int, align 8
   %b36i2 = alloca %struct.Int_Int, align 8
   %v_4 = alloca %struct.Int_Int, align 8
   %b30i1 = alloca %struct.Int_Int, align 8
   %b28i2 = alloca %struct.Int_Int, align 8
   %v_3 = alloca %struct.Int_Int, align 8
   %b23i2 = alloca %struct.Int_Int, align 8
   %v_2 = alloca %struct.Int_Int, align 8
   %b21i2 = alloca %struct.Int_Int, align 8
   %v_1 = alloca %struct.Int_Int, align 8
   %b16i2 = alloca %struct.Bool_Bool, align 4
   %b16i1 = alloca %struct.String_String, align 8
   %b13i1 = alloca %struct.Int_Int, align 8
   %b11i1 = alloca %struct.Int_Int, align 8
   %b1i2 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.Int_Int, align 8
   %match_var_0 = alloca %struct.Int_Int, align 8
   %b0i7 = alloca %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, align 8
   %b0i6 = alloca %struct.Main_Foo, align 4
   %b0i4 = alloca %struct.Bool_Bool, align 4
   %b0i3 = alloca %struct.String_String, align 8
   %b0i2 = alloca %struct.Int_Int, align 8
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %b0i1)
   %tmp_b0i2_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i2, i32 0, i32 0
   store i64 3, ptr %tmp_b0i2_1, align 8
   %tmp_b0i3_1 = getelementptr inbounds %struct.String_String, ptr %b0i3, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b0i3_1, align 8
   %tmp_b0i3_2 = getelementptr inbounds %struct.String_String, ptr %b0i3, i32 0, i32 1
   store i64 1, ptr %tmp_b0i3_2, align 8
   call void @Bool_Bool_True(ptr %b0i4)
   call void @Main_Foo_Bar(ptr %b0i4, ptr %b0i6)
   call void @siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo(ptr %b0i1, ptr %b0i2, ptr %b0i3, ptr %b0i6, ptr %b0i7)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b1i1, ptr align 8 %match_var_0, i64 8, i1 false)
   call void @siko_Tuple_(ptr %b1i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i2, i64 0, i1 false)
   ret void
block2:
   %b2i1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %b0i7, i32 0, i32 0
   %b2i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %b0i7, i32 0, i32 1
   %b2i3 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %b0i7, i32 0, i32 2
   %b2i4 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %b0i7, i32 0, i32 3
   br label %block3
block3:
   %tmp_switch_var_block3_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b2i1, i32 0, i32 0
   %tmp_switch_var_block3_2 = load i32, ptr %tmp_switch_var_block3_1, align 4
   switch i32 %tmp_switch_var_block3_2, label %block4 [
i32 1, label %block14
]

block4:
   br label %block5
block5:
   %tmp_switch_var_block5_1 = getelementptr inbounds %struct.Int_Int, ptr %b2i2, i32 0, i32 0
   %tmp_switch_var_block5_2 = load i64, ptr %tmp_switch_var_block5_1, align 8
   switch i64 %tmp_switch_var_block5_2, label %block6 [

]

block6:
   br label %block7
block7:
   %tmp_switch_var_block7_1 = getelementptr inbounds %struct.Main_Foo, ptr %b2i4, i32 0, i32 0
   %tmp_switch_var_block7_2 = load i32, ptr %tmp_switch_var_block7_1, align 4
   switch i32 %tmp_switch_var_block7_2, label %block8 [

]

block8:
   %tmp_b8i1_1 = bitcast %struct.Main_Foo* %b2i4 to %struct.Main_Foo_Bar*
   %b8i1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_b8i1_1, i32 0, i32 1
   %b8i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b8i1, i32 0, i32 0
   br label %block9
block9:
   %tmp_switch_var_block9_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b8i2, i32 0, i32 0
   %tmp_switch_var_block9_2 = load i32, ptr %tmp_switch_var_block9_1, align 4
   switch i32 %tmp_switch_var_block9_2, label %block10 [
i32 1, label %block12
]

block10:
   br label %block11
block11:
   %tmp_b11i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b11i1, i32 0, i32 0
   store i64 3, ptr %tmp_b11i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b11i1, i64 8, i1 false)
   br label %block1
block12:
   br label %block13
block13:
   %tmp_b13i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b13i1, i32 0, i32 0
   store i64 3, ptr %tmp_b13i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b13i1, i64 8, i1 false)
   br label %block1
block14:
   br label %block15
block15:
   %tmp_switch_var_block15_1 = getelementptr inbounds %struct.Int_Int, ptr %b2i2, i32 0, i32 0
   %tmp_switch_var_block15_2 = load i64, ptr %tmp_switch_var_block15_1, align 8
   switch i64 %tmp_switch_var_block15_2, label %block31 [
i64 1, label %block16
]

block16:
   %tmp_b16i1_1 = getelementptr inbounds %struct.String_String, ptr %b16i1, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_b16i1_1, align 8
   %tmp_b16i1_2 = getelementptr inbounds %struct.String_String, ptr %b16i1, i32 0, i32 1
   store i64 3, ptr %tmp_b16i1_2, align 8
   call void @String_String_eq(ptr %b2i3, ptr %b16i1, ptr %b16i2)
   %tmp_switch_var_block16_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b16i2, i32 0, i32 0
   %tmp_switch_var_block16_2 = load i32, ptr %tmp_switch_var_block16_1, align 4
   switch i32 %tmp_switch_var_block16_2, label %block17 [
i32 1, label %block24
]

block17:
   %tmp_switch_var_block17_1 = getelementptr inbounds %struct.Main_Foo, ptr %b2i4, i32 0, i32 0
   %tmp_switch_var_block17_2 = load i32, ptr %tmp_switch_var_block17_1, align 4
   switch i32 %tmp_switch_var_block17_2, label %block18 [

]

block18:
   %tmp_b18i1_1 = bitcast %struct.Main_Foo* %b2i4 to %struct.Main_Foo_Bar*
   %b18i1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_b18i1_1, i32 0, i32 1
   %b18i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b18i1, i32 0, i32 0
   br label %block19
block19:
   %tmp_switch_var_block19_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b18i2, i32 0, i32 0
   %tmp_switch_var_block19_2 = load i32, ptr %tmp_switch_var_block19_1, align 4
   switch i32 %tmp_switch_var_block19_2, label %block20 [
i32 1, label %block22
]

block20:
   br label %block21
block21:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_1, ptr align 8 %b2i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b21i2, ptr align 8 %v_1, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b21i2, i64 8, i1 false)
   br label %block1
block22:
   br label %block23
block23:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_2, ptr align 8 %b2i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b23i2, ptr align 8 %v_2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b23i2, i64 8, i1 false)
   br label %block1
block24:
   %tmp_switch_var_block24_1 = getelementptr inbounds %struct.Main_Foo, ptr %b2i4, i32 0, i32 0
   %tmp_switch_var_block24_2 = load i32, ptr %tmp_switch_var_block24_1, align 4
   switch i32 %tmp_switch_var_block24_2, label %block25 [

]

block25:
   %tmp_b25i1_1 = bitcast %struct.Main_Foo* %b2i4 to %struct.Main_Foo_Bar*
   %b25i1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_b25i1_1, i32 0, i32 1
   %b25i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b25i1, i32 0, i32 0
   br label %block26
block26:
   %tmp_switch_var_block26_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b25i2, i32 0, i32 0
   %tmp_switch_var_block26_2 = load i32, ptr %tmp_switch_var_block26_1, align 4
   switch i32 %tmp_switch_var_block26_2, label %block27 [
i32 1, label %block29
]

block27:
   br label %block28
block28:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_3, ptr align 8 %b2i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b28i2, ptr align 8 %v_3, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b28i2, i64 8, i1 false)
   br label %block1
block29:
   br label %block30
block30:
   %tmp_b30i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b30i1, i32 0, i32 0
   store i64 1, ptr %tmp_b30i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b30i1, i64 8, i1 false)
   br label %block1
block31:
   br label %block32
block32:
   %tmp_switch_var_block32_1 = getelementptr inbounds %struct.Main_Foo, ptr %b2i4, i32 0, i32 0
   %tmp_switch_var_block32_2 = load i32, ptr %tmp_switch_var_block32_1, align 4
   switch i32 %tmp_switch_var_block32_2, label %block33 [

]

block33:
   %tmp_b33i1_1 = bitcast %struct.Main_Foo* %b2i4 to %struct.Main_Foo_Bar*
   %b33i1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_b33i1_1, i32 0, i32 1
   %b33i2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %b33i1, i32 0, i32 0
   br label %block34
block34:
   %tmp_switch_var_block34_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b33i2, i32 0, i32 0
   %tmp_switch_var_block34_2 = load i32, ptr %tmp_switch_var_block34_1, align 4
   switch i32 %tmp_switch_var_block34_2, label %block35 [
i32 1, label %block37
]

block35:
   br label %block36
block36:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_4, ptr align 8 %b2i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b36i2, ptr align 8 %v_4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b36i2, i64 8, i1 false)
   br label %block1
block37:
   br label %block38
block38:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_5, ptr align 8 %b2i2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b38i2, ptr align 8 %v_5, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_0, ptr align 8 %b38i2, i64 8, i1 false)
   br label %block1
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_Foo_Bar(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Foo_Bar, align 4
   %tag = getelementptr inbounds %struct.Main_Foo_Bar, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


